use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use tracing::instrument;
use warp::http::StatusCode;
use warp::{Rejection, Reply};

use rust_core::entities::question::{QuestionEntity, QuestionId};
use rust_core::entities::question_filter::QuestionFilter;
use rust_core::ports::question::QuestionPort;

use grpc_client::grpc_client::gpt_answer::GptAnswerGrpcClient;

use crate::errors::WarpError;

/// Handler for retrieving questions based on query parameters.
///
/// This function retrieves questions based on the provided query parameters. It takes a HashMap
/// containing the query parameters and a reference to the QuestionPort trait object. It returns
/// a JSON response containing the list of questions.
#[instrument(level = "info", skip(question_port))]
pub async fn get_questions(
    question_port: Arc<dyn QuestionPort + Send + Sync>,
    query: HashMap<String, String>,
) -> Result<impl Reply, Rejection> {
    let question_filter = QuestionFilter::from_query(&query).map_err(WarpError)?;
    let questions = question_port
        .list(&question_filter)
        .await
        .map_err(WarpError)?;
    Ok(warp::reply::json(&questions))
}

/// Handler for retrieving a question by ID.
///
/// This function retrieves a question with the specified ID from the system. It takes the ID of
/// the question to be got as a string and a reference to the QuestionPort trait object. It returns
/// a JSON response containing the question.
#[instrument(level = "info", skip(question_port))]
pub async fn get_question(
    question_port: Arc<dyn QuestionPort + Send + Sync>,
    id: String,
) -> Result<impl Reply, Rejection> {
    let question = question_port
        .get(&QuestionId::from_str(id.as_str()).unwrap())
        .await
        .map_err(WarpError)?;
    Ok(warp::reply::json(&question))
}

/// Handler for adding a new question.
///
/// This function adds a new question to the system. It takes a QuestionEntity representing the
/// question to be added and a reference to the QuestionPort trait object. It returns a success
/// response with status code 200 if the question is added successfully.
#[instrument(level = "info", skip(question_port))]
pub async fn add_question(
    question_port: Arc<dyn QuestionPort + Send + Sync>,
    question: QuestionEntity,
) -> Result<impl Reply, Rejection> {
    question_port.add(question).await.map_err(WarpError)?;
    Ok(warp::reply::with_status("Question added", StatusCode::OK))
}

/// Handler for deleting a question by ID.
///
/// This function deletes a question with the specified ID from the system. It takes the ID of
/// the question to be deleted as a string and a reference to the QuestionPort trait object. It
/// returns a success response with status code 200 if the question is deleted successfully.
#[instrument(level = "info", skip(question_port))]
pub async fn delete_question(
    question_port: Arc<dyn QuestionPort + Send + Sync>,
    id: String,
) -> Result<impl Reply, Rejection> {
    question_port
        .delete(&QuestionId::from_str(id.as_str()).unwrap())
        .await
        .map_err(WarpError)?;
    Ok(warp::reply::with_status("Question deleted", StatusCode::OK))
}

/// Handler for updating a question by ID.
///
/// This function updates a question with the specified ID in the system. It takes the ID of the
/// question to be updated as a string, the updated QuestionEntity, and a reference to the
/// QuestionPort trait object. It returns a success response with status code 200 if the question
/// is updated successfully.
#[instrument(level = "info", skip(question_port))]
pub async fn update_question(
    question_port: Arc<dyn QuestionPort + Send + Sync>,
    id: String,
    mut question: QuestionEntity,
) -> Result<impl Reply, Rejection> {
    question.id = QuestionId::from_str(id.as_str()).unwrap();
    question_port.update(question).await.map_err(WarpError)?;
    Ok(warp::reply::with_status("Question updated", StatusCode::OK))
}

/// Controller for handling HTTP GET requests to fetch answers for a given question ID.
///
/// This controller retrieves a question from the provided `QuestionPort` based on the
/// specified ID, calls the gRPC client (`GrpcClient`) to get an answer using the
/// question's text, and responds with the answer in a JSON format.
///
/// # Arguments
///
/// * `question_port`: A trait object implementing `QuestionPort` for interacting with questions.
/// * `gpt_client`: An instance of `GrpcClient` used for communication with the gRPC server.
/// * `id`: The ID of the question to fetch the answer for.
///
/// # Returns
///
/// Returns a `Result` containing the HTTP response. If successful, responds with a String
/// representation of the answer and a status code of 200 OK. If there's an error during
/// question retrieval, gRPC communication, or response construction, it returns a Warp Rejection.
#[instrument(level = "info", skip(question_port))]
pub async fn get_question_answer_controller(
    question_port: Arc<dyn QuestionPort + Send + Sync>,
    id: String,
) -> Result<impl Reply, Rejection> {
    let question = question_port
        .get(&QuestionId::from_str(id.as_str()).unwrap())
        .await
        .map_err(WarpError)?;

    let client = GptAnswerGrpcClient::get_instance().await;

    let answer = client
        .unwrap()
        .get_answer(&question.content)
        .await
        .map_err(|err| WarpError(err))?;

    Ok(warp::reply::with_status(answer, StatusCode::OK))
}
