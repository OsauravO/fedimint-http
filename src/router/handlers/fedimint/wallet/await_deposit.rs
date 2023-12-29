use crate::{error::AppError, state::AppState};
use anyhow::anyhow;
use axum::{extract::State, http::StatusCode, Json};
use fedimint_core::core::OperationId;
use fedimint_wallet_client::{DepositState, WalletClientModule};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AwaitDepositRequest {
    pub operation_id: OperationId,
}

#[derive(Debug, Serialize)]
pub struct AwaitDepositResponse {
    pub status: DepositState,
}

#[axum_macros::debug_handler]
pub async fn handle_await_deposit(
    State(state): State<AppState>,
    Json(req): Json<AwaitDepositRequest>,
) -> Result<Json<AwaitDepositResponse>, AppError> {
    let mut updates = state
        .fm
        .get_first_module::<WalletClientModule>()
        .subscribe_deposit_updates(req.operation_id)
        .await?
        .into_stream();

    while let Some(update) = updates.next().await {
        match update {
            DepositState::Confirmed(tx) => {
                return Ok(Json(AwaitDepositResponse {
                    status: DepositState::Confirmed(tx),
                }))
            }
            DepositState::Claimed(tx) => {
                return Ok(Json(AwaitDepositResponse {
                    status: DepositState::Claimed(tx),
                }))
            }
            DepositState::Failed(reason) => {
                return Err(AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    anyhow!(reason),
                ))
            }
            _ => {}
        }
    }

    Err(AppError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        anyhow!("Unexpected end of stream"),
    ))
}