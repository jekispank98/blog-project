use std::sync::Arc;
use tonic::{Request, Response, Status};
use crate::blog::{
    blog_service_server::BlogService as ProtoBlogService,
    CreatePostRequest, GetPostRequest, UpdatePostRequest, DeletePostRequest, ListPostsRequest,
    PostResponse, DeletePostResponse, ListPostsResponse, Post as ProtoPost,
};
use crate::handlers::{BlogService, AuthService};
use crate::infrastructure::jwt::Jwt;
use crate::domain::error::ParserError;

pub struct BlogGrpcService {
    blog_service: Arc<BlogService>,
    _auth_service: Arc<AuthService>,
    jwt_service: Arc<Jwt>,
}

impl BlogGrpcService {
    pub fn new(
        blog_service: Arc<BlogService>,
        auth_service: Arc<AuthService>,
        jwt_service: Arc<Jwt>,
    ) -> Self {
        Self {
            blog_service,
            _auth_service: auth_service,
            jwt_service,
        }
    }

    fn authenticate<T>(&self, request: &Request<T>) -> Result<String, Status> {
        let auth_header = request
            .metadata()
            .get("authorization")
            .ok_or_else(|| Status::unauthenticated("Missing authorization header"))?
            .to_str()
            .map_err(|_| Status::unauthenticated("Invalid authorization header format"))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(Status::unauthenticated("Invalid token format"));
        }

        let token = &auth_header[7..];
        let claims = self.jwt_service.verify_token(token)
            .map_err(|_| Status::unauthenticated("Invalid or expired token"))?;

        Ok(claims.user_id)
    }

    fn map_error(e: ParserError) -> Status {
        match e {
            ParserError::PostNotFound => Status::not_found("Post not found"),
            ParserError::Forbidden => Status::permission_denied("Forbidden"),
            ParserError::UserNotFound => Status::not_found("User not found"),
            ParserError::UserAlreadyExists => Status::already_exists("User already exists"),
            ParserError::InvalidCredentials => Status::unauthenticated("Invalid credentials"),
            ParserError::DatabaseError(err) => Status::internal(format!("Database error: {}", err)),
            ParserError::InternalError(err) => Status::internal(format!("Internal error: {}", err)),
        }
    }
}

#[tonic::async_trait]
impl ProtoBlogService for BlogGrpcService {
    async fn create_post(
        &self,
        request: Request<CreatePostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        let user_id = self.authenticate(&request)?;
        let req = request.into_inner();

        let post = self.blog_service
            .create_post(req.title, req.content, user_id)
            .await
            .map_err(Self::map_error)?;

        Ok(Response::new(PostResponse {
            post: Some(ProtoPost {
                post_id: post.id,
                title: post.title,
                content: post.content,
                user_id: post.author_id,
                created_at: post.created_at.to_string(),
            }),
        }))
    }

    async fn get_post(
        &self,
        request: Request<GetPostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        let req = request.into_inner();

        let post = self.blog_service
            .get_post(req.post_id)
            .await
            .map_err(Self::map_error)?;

        Ok(Response::new(PostResponse {
            post: Some(ProtoPost {
                post_id: post.id,
                title: post.title,
                content: post.content,
                user_id: post.author_id,
                created_at: post.created_at.to_string(),
            }),
        }))
    }

    async fn update_post(
        &self,
        request: Request<UpdatePostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        let user_id = self.authenticate(&request)?;
        let req = request.into_inner();

        let post = self.blog_service
            .update_post(req.post_id, req.title, req.content, user_id)
            .await
            .map_err(Self::map_error)?;

        Ok(Response::new(PostResponse {
            post: Some(ProtoPost {
                post_id: post.id,
                title: post.title,
                content: post.content,
                user_id: post.author_id,
                created_at: post.created_at.to_string(),
            }),
        }))
    }

    async fn delete_post(
        &self,
        request: Request<DeletePostRequest>,
    ) -> Result<Response<DeletePostResponse>, Status> {
        let user_id = self.authenticate(&request)?;
        let req = request.into_inner();

        self.blog_service
            .delete_post(req.post_id, user_id)
            .await
            .map_err(Self::map_error)?;

        Ok(Response::new(DeletePostResponse {
            success: true,
            message: "Post deleted successfully".to_string(),
        }))
    }

    async fn list_posts(
        &self,
        request: Request<ListPostsRequest>,
    ) -> Result<Response<ListPostsResponse>, Status> {
        let req = request.into_inner();
        
        let limit = if req.page_size <= 0 { 10 } else { req.page_size as i64 };
        let offset = if req.page <= 1 { 0 } else { (req.page as i64 - 1) * limit };

        let result = self.blog_service
            .list_posts(limit, offset)
            .await
            .map_err(Self::map_error)?;

        let proto_posts = result.posts.into_iter().map(|p| ProtoPost {
            post_id: p.id,
            title: p.title,
            content: p.content,
            user_id: p.author_id,
            created_at: p.created_at.to_string(),
        }).collect();

        Ok(Response::new(ListPostsResponse {
            posts: proto_posts,
            total_count: result.total as i32,
        }))
    }
}
