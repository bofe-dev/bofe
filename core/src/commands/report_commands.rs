#[cfg(feature = "graphql")]
use std::net::IpAddr;

use uuid::Uuid;

#[cfg(feature = "graphql")]
use validator::Validate;

use crate::db_pool;
use crate::enums::ReportTarget;
use crate::models::Report;

#[cfg(feature = "graphql")]
use crate::constants::ERROR_IS_INVALID;
#[cfg(feature = "graphql")]
use crate::jobs_storage;
#[cfg(feature = "graphql")]
use crate::models::User;
#[cfg(feature = "graphql")]
use crate::params::ReportParams;

#[cfg(feature = "graphql")]
use super::{OrValidationErrors, ValidationResult, get_board_by_id, get_card_by_id, get_user_by_id};

pub async fn get_report_by_id<'a>(id: Uuid) -> sqlx::Result<Report<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Report,
        r#"SELECT
            id,
            user_id,
            ip_address,
            target AS "target!: ReportTarget",
            target_id,
            data,
            message,
            reviewed_at,
            created_at,
            updated_at
        FROM reports WHERE id = $1 LIMIT 1"#,
        id, // $1
    )
    .fetch_one(db_pool)
    .await
}

#[cfg(feature = "graphql")]
pub(crate) async fn insert_report<'a>(
    user: &User<'_>,
    ip_address: &IpAddr,
    params: ReportParams,
) -> ValidationResult<Report<'a>> {
    params.validate()?;

    let db_pool = db_pool().await;

    let data = match params.target {
        ReportTarget::Board => get_board_by_id(params.target_id)
            .await
            .ok()
            .and_then(|board| serde_json::to_value(board).ok()),
        ReportTarget::Card => get_card_by_id(params.target_id)
            .await
            .ok()
            .and_then(|card| serde_json::to_value(card).ok()),
        ReportTarget::User => get_user_by_id(params.target_id)
            .await
            .ok()
            .and_then(|user| serde_json::to_value(user).ok()),
    }
    .or_validation_errors_with("target_id", ERROR_IS_INVALID.clone())?;

    let message = params.message.trim();

    let report = sqlx::query_as!(
        Report,
        r#"INSERT INTO reports (user_id, ip_address, target, target_id, data, message) VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING
            id,
            user_id,
            ip_address,
            target AS "target!: ReportTarget",
            target_id,
            data,
            message,
            reviewed_at,
            created_at,
            updated_at"#,
        user.id,                // $1
        ip_address.to_string(), // $2
        params.target as _,     // $3
        params.target_id,       // $4
        data,                   // $5
        message,                // $6
    )
    .fetch_one(db_pool)
    .await
    .or_validation_errors()?;

    jobs_storage().await.push_new_report(&report).await;

    Ok(report)
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{fake_ip_addr, fake_paragraph, insert_test_board, insert_test_report, insert_test_user};

    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn get_report_by_id_with_valid_id_returns_ok() {
        let report = insert_test_report().await;

        let result = get_report_by_id(report.id).await;

        assert!(result.is_ok());

        let recd_report = result.unwrap();

        assert_eq!(recd_report.id, report.id);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert_report_with_valid_params_returns_ok() {
        let user = insert_test_user(None).await;
        let board = insert_test_board(None).await;
        let ip_address = fake_ip_addr();
        let message = fake_paragraph();

        let result = insert_report(
            &user,
            &ip_address,
            ReportParams {
                target: ReportTarget::Board,
                target_id: board.id,
                message: message.clone(),
            },
        )
        .await;

        assert!(result.is_ok());

        let report = result.unwrap();

        assert_eq!(report.user_id, user.id);
        assert_eq!(report.ip_address, ip_address.to_string());
        assert_eq!(report.target.to_string(), ReportTarget::Board.to_string());
        assert_eq!(report.target_id, board.id);
        assert_eq!(report.message, message);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert_report_with_empty_message_returns_err() {
        let user = insert_test_user(None).await;
        let board = insert_test_board(None).await;
        let ip_address = fake_ip_addr();

        let result = insert_report(
            &user,
            &ip_address,
            ReportParams {
                target: ReportTarget::Board,
                target_id: board.id,
                message: String::new(),
            },
        )
        .await;

        assert!(result.is_err());
    }
}
