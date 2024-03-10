use gpt_answer::gpt_answer_service_server::{GptAnswerService, GptAnswerServiceServer};
use gpt_answer::{GetAnswerPayload, GetAnswerResponse};
use tonic::{transport::Server, Request, Response, Status};

mod gpt_answer {
    tonic::include_proto!("gpt_answer");
}

#[derive(Debug, Default)]
pub struct GptAnswerServer;

#[tonic::async_trait]
impl GptAnswerService for GptAnswerServer {
    async fn get_answer(
        &self,
        request: Request<GetAnswerPayload>,
    ) -> Result<Response<GetAnswerResponse>, Status> {
        let payload = request.into_inner();
        // TODO: Implement your logic to generate an answer based on the question.
        let answer = format!("Answer to: {}", payload.question);

        let response = GetAnswerResponse { answer };
        Ok(Response::new(response))
    }
}

pub async fn init_gpt_answer_server() {
    let result = "http://0.0.0.0:50051".parse().map_err(|err| {
        println!("Error: {:?}", err);
    });

    if result.is_ok() {
        let addr = result.unwrap();

        println!("GPT Answer server config at {}", addr);

        let gpt_answer_server = GptAnswerServer::default();

        Server::builder()
            .add_service(GptAnswerServiceServer::new(gpt_answer_server))
            .serve(addr)
            .await
            .unwrap();

        println!("GPT Answer server started at {}", addr);
    } else {
        println!("GPT Answer server failed to start");
    }
}
