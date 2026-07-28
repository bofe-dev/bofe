use std::borrow::Cow;

#[cfg(feature = "graphql")]
use std::path::PathBuf;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "graphql")]
use url::Url;

#[cfg(feature = "graphql")]
use crate::enums::BlobFileType;
use crate::enums::{CountryCode, LanguageCode};

#[cfg(feature = "graphql")]
use crate::commands;
#[cfg(feature = "graphql")]
use crate::config::STORAGE_CONFIG;

#[cfg(feature = "graphql")]
use super::Attachment;

#[derive(Clone, Deserialize, Serialize)]
pub struct User<'a> {
    pub id: Uuid,
    pub username: Cow<'a, str>,
    pub email: Cow<'a, str>,
    pub email_confirmed_at: Option<DateTime<Utc>>,
    pub(crate) encrypted_password: Cow<'a, str>,
    pub full_name: Cow<'a, str>,
    pub display_name: Cow<'a, str>,
    pub birthdate: NaiveDate,
    pub language_code: LanguageCode,
    pub country_code: CountryCode,
    pub avatar_image_attachment_id: Option<Uuid>,
    pub disabled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[cfg(feature = "graphql")]
impl User<'_> {
    pub async fn avatar_image(&self, size: u16) -> anyhow::Result<Vec<u8>> {
        if let Ok(Some(attachment)) = self.avatar_image_attachment().await {
            attachment.blob().await?.read_thumbnail(size, size)
        } else {
            commands::get_user_text_icon(self, size)
        }
    }

    pub(crate) fn text_icon_path(&self, size: u16) -> PathBuf {
        STORAGE_CONFIG
            .path
            .join(format!("users/{}/text-icon/{size}x{size}.jpg", self.id))
    }

    pub fn avatar_image_url(&self, size: u16) -> Url {
        STORAGE_CONFIG
            .url
            .join(&format!(
                "users/{}/avatar-image?size={size}&timestamp={}",
                self.id,
                self.updated_at.unwrap_or(self.created_at).timestamp()
            ))
            .unwrap()
    }

    pub async fn avatar_image_attachment(&self) -> sqlx::Result<Option<Attachment<'_>>> {
        if let Some(attachment_id) = self.avatar_image_attachment_id {
            commands::get_attachment_by_id(attachment_id).await.map(Some)
        } else {
            Ok(None)
        }
    }

    pub async fn avatar_image_file_type(&self) -> anyhow::Result<BlobFileType> {
        if let Ok(Some(attachment)) = self.avatar_image_attachment().await {
            Ok(attachment.blob().await?.thumbnail_file_type())
        } else {
            Ok(BlobFileType::ImageJpeg)
        }
    }

    pub(crate) fn email_is_confirmed(&self) -> bool {
        self.email_confirmed_at.is_some()
    }

    pub(crate) fn initials(&self) -> String {
        self.username[0..2].to_uppercase()
    }

    pub(crate) fn verify_password(&self, password: &str) -> bool {
        commands::verify_password(&self.encrypted_password, password)
    }
}
