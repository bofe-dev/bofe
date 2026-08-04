use std::net::IpAddr;

use chrono::{DateTime, NaiveDate, Utc};
use fake::Fake;
use fake::faker::chrono::en::DateTimeBefore;
use fake::faker::internet::en::{FreeEmail, IP, Password, Username};
use fake::faker::lorem::en::Paragraph;
use fake::faker::name::en::Name;
use rand::rng;
use rand::rngs::ThreadRng;

use crate::commands;
use crate::enums::{BoardVisibility, CountryCode, LanguageCode, ReportTarget};
use crate::models::{Board, Card, List, Report, User};
use crate::params::{BoardParams, CardParams, ListParams, ReportParams, UserParams};

pub fn fake_birthdate() -> NaiveDate {
    DateTimeBefore(Utc::now()).fake::<DateTime<Utc>>().date_naive()
}

pub fn fake_country_code() -> CountryCode {
    CountryCode::VE
}

pub fn fake_email() -> String {
    FreeEmail().fake_with_rng(&mut rng())
}

pub fn fake_ip_addr() -> IpAddr {
    IP().fake_with_rng(&mut rng())
}

pub fn fake_language_code() -> LanguageCode {
    LanguageCode::Es
}

pub fn fake_name() -> String {
    let mut name: String = Name().fake_with_rng(&mut rng());

    name.truncate(255);

    name
}

pub fn fake_paragraph() -> String {
    Paragraph(1..5).fake()
}

pub fn fake_password() -> String {
    Password(6..129).fake()
}

pub fn fake_slug() -> String {
    Username()
        .fake_with_rng::<String, ThreadRng>(&mut rng())
        .replace("_", "-")
        .replace(".", "-")
}

pub fn fake_username() -> String {
    let mut username: String = Username().fake_with_rng(&mut rng());

    username.truncate(16);

    username
}

pub async fn insert_test_board<'a>(user: Option<&User<'_>>) -> Board<'a> {
    let user = if let Some(user) = user {
        user.clone()
    } else {
        insert_test_user(None).await
    };

    let name = fake_name();
    let slug = fake_slug();
    let description = fake_paragraph();

    commands::insert_board(
        &user,
        BoardParams {
            background_image_attachment_id: None,
            name,
            slug,
            description,
            visibility: BoardVisibility::Private,
        },
    )
    .await
    .expect("Could not insert board")
}

pub async fn insert_test_card<'a>(is_archived: bool) -> Card<'a> {
    let user = insert_test_user(None).await;
    let list = insert_test_list(Some(&user), None).await;
    let content = fake_paragraph();

    let card = commands::insert_card(
        &user,
        CardParams {
            list_id: list.id,
            cover_image_attachment_id: None,
            content,
            attachment_ids: vec![],
            label_ids: vec![],
        },
    )
    .await
    .expect("Could not insert card");

    if is_archived {
        return commands::archive_card(&user, &card)
            .await
            .expect("Could not archive card");
    }

    card
}

pub async fn insert_test_list<'a>(user: Option<&User<'_>>, board: Option<&Board<'_>>) -> List<'a> {
    let user = if let Some(user) = user {
        user.clone()
    } else {
        insert_test_user(None).await
    };

    let board = if let Some(board) = board {
        board.clone()
    } else {
        insert_test_board(Some(&user)).await
    };

    let name = fake_name();

    commands::insert_list(
        &user,
        ListParams {
            board_id: board.id,
            name,
            archive_cards: false,
        },
    )
    .await
    .expect("Could not insert list")
}

pub async fn insert_test_report<'a>() -> Report<'a> {
    let user = insert_test_user(None).await;
    let board = insert_test_board(None).await;
    let ip_address = fake_ip_addr();
    let message = fake_paragraph();

    commands::insert_report(
        &user,
        &ip_address,
        ReportParams {
            target: ReportTarget::Board,
            target_id: board.id,
            message,
        },
    )
    .await
    .expect("Could not insert report")
}

pub async fn insert_test_user<'a>(password: Option<&'_ str>) -> User<'a> {
    let username = fake_username();
    let email = fake_email();

    let password = if let Some(password) = password {
        password.to_owned()
    } else {
        fake_password()
    };

    let full_name = fake_name();
    let birthdate = fake_birthdate();
    let language_code = fake_language_code();
    let country_code = fake_country_code();

    commands::insert_user(UserParams {
        username,
        email,
        password,
        full_name,
        birthdate,
        language_code: Some(language_code),
        country_code,
    })
    .await
    .expect("Could not insert user")
}
