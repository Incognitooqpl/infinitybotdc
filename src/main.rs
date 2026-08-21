use serenity::{
    all::*,
    async_trait,
    builder::{
        CreateButton,
        CreateEmbed,
        CreateInteractionResponse,
        CreateInteractionResponseMessage,
        CreateMessage,
    },
    client::{Context, EventHandler},
};

use std::{
    convert::Infallible,
    env,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

// =====================================================
// KONFIGURACJA
// =====================================================

const PREFIX: &str = "!";
const VERIFY_BUTTON_ID: &str = "infinitystudio_verify";
const MEMBER_ROLE_NAME: &str = "Member";

// =====================================================
// HEALTH SERVER
// =====================================================
//
// Render wymaga publicznego portu.
// UptimeRobot będzie sprawdzał:
// https://TWOJ-BOT.onrender.com/health
//
// =====================================================

async fn health_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port = env::var("PORT")
        .unwrap_or_else(|_| "10000".to_string());

    let address = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&address).await?;

    println!("=================================");
    println!("🌐 Health Server");
    println!("=================================");
    println!("Listening on: {}", address);
    println!("Health endpoint: /health");
    println!("=================================");

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buffer = [0u8; 4096];

            let bytes_read =
                match socket.read(&mut buffer).await {
                    Ok(bytes) => bytes,
                    Err(_) => return,
                };

            if bytes_read == 0 {
                return;
            }

            let request =
                String::from_utf8_lossy(
                    &buffer[..bytes_read]
                );

            let path = request
                .lines()
                .next()
                .and_then(|line| {
                    line.split_whitespace().nth(1)
                })
                .unwrap_or("/");

            let (status, body) =
                if path == "/health" {

                    (
                        "200 OK",
                        r#"{"status":"ok","service":"infinitystudio-bot"}"#,
                    )

                } else if path == "/" {

                    (
                        "200 OK",
                        r#"{"status":"online","service":"∞studio Bot"}"#,
                    )

                } else {

                    (
                        "404 Not Found",
                        r#"{"status":"not_found"}"#,
                    )
                };

            let response = format!(
                "HTTP/1.1 {}\r\n\
                 Content-Type: application/json\r\n\
                 Content-Length: {}\r\n\
                 Connection: close\r\n\
                 \r\n\
                 {}",
                status,
                body.len(),
                body
            );

            let _ =
                socket
                    .write_all(
                        response.as_bytes()
                    )
                    .await;

            let _ =
                socket
                    .shutdown()
                    .await;
        });
    }
}

// =====================================================
// DISCORD HANDLER
// =====================================================

struct Handler;

impl Handler {

    // =================================================
    // STAFF CHECK
    // =================================================

    fn is_staff(member: &Member) -> bool {
        member
            .permissions
            .unwrap_or_default()
            .contains(
                Permissions::ADMINISTRATOR
            )
    }

    // =================================================
    // BRAK UPRAWNIEŃ
    // =================================================

    async fn no_permission(
        ctx: &Context,
        msg: &Message,
    ) {
        let _ =
            msg.channel_id
                .say(
                    &ctx.http,
                    "❌ Nie masz uprawnień \
                     do użycia tej komendy.",
                )
                .await;
    }

    // =================================================
    // EPHEMERAL RESPONSE
    // =================================================

    async fn ephemeral(
        ctx: &Context,
        interaction: &ComponentInteraction,
        content: &str,
    ) {
        let _ =
            interaction
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(content)
                            .ephemeral(true),
                    ),
                )
                .await;
    }
}

// =====================================================
// EVENT HANDLER
// =====================================================

#[async_trait]
impl EventHandler for Handler {

    // =================================================
    // READY
    // =================================================

    async fn ready(
        &self,
        ctx: Context,
        ready: Ready,
    ) {
        println!("=================================");
        println!("∞studio Bot");
        println!("=================================");
        println!(
            "Zalogowano jako: {}",
            ready.user.name
        );
        println!(
            "ID: {}",
            ready.user.id
        );
        println!("Status: DND");
        println!("Playing: InfiniteCraft");
        println!("=================================");

        ctx.set_presence(
            Some(
                ActivityData::playing(
                    "InfiniteCraft"
                )
            ),
            OnlineStatus::DoNotDisturb,
        );
    }

    // =================================================
    // MESSAGE
    // =================================================

    async fn message(
        &self,
        ctx: Context,
        msg: Message,
    ) {

        // ---------------------------------------------
        // IGNORUJ BOTY
        // ---------------------------------------------

        if msg.author.bot {
            return;
        }

        // ---------------------------------------------
        // PREFIX
        // ---------------------------------------------

        if !msg.content.starts_with(PREFIX) {
            return;
        }

        let content =
            msg.content[PREFIX.len()..]
                .trim();

        if content.is_empty() {
            return;
        }

        // ---------------------------------------------
        // PARSOWANIE
        // ---------------------------------------------

        let mut parts =
            content.splitn(2, ' ');

        let command =
            parts
                .next()
                .unwrap_or("")
                .to_lowercase();

        let args =
            parts
                .next()
                .unwrap_or("")
                .trim();

        // =================================================
        // !infinitecraft
        // PUBLIC
        // =================================================

        if command == "infinitecraft" {

            let embed =
                CreateEmbed::new()
                    .title("🎮 InfiniteCraft")
                    .description(
                        "❌ Twoje konto Discord \
                         nie jest jeszcze \
                         połączone z InfiniteCraft."
                    )
                    .footer(
                        CreateEmbedFooter::new(
                            "∞studio • InfiniteCraft"
                        )
                    );

            let _ =
                msg.channel_id
                    .send_message(
                        &ctx.http,
                        CreateMessage::new()
                            .embed(embed),
                    )
                    .await;

            return;
        }

        // =================================================
        // !verify
        // PUBLIC
        // =================================================

        if command == "verify" {

            let embed =
                CreateEmbed::new()
                    .title(
                        "∞studio Verification"
                    )
                    .description(
                        "## Weryfikacja serwera\n\n\
                         Kliknij przycisk **Verify**, \
                         aby otrzymać rangę `Member`.\n\n\
                         Po pomyślnej weryfikacji \
                         uzyskasz dostęp do kanałów \
                         dla członków ∞studio."
                    )
                    .footer(
                        CreateEmbedFooter::new(
                            "∞studio • Verification System"
                        )
                    );

            let button =
                CreateButton::new(
                    VERIFY_BUTTON_ID
                )
                .label("Verify")
                .emoji('✅')
                .style(
                    ButtonStyle::Success
                );

            let _ =
                msg.channel_id
                    .send_message(
                        &ctx.http,
                        CreateMessage::new()
                            .embed(embed)
                            .button(button),
                    )
                    .await;

            return;
        }

        // =================================================
        // OD TEGO MIEJSCA STAFF ONLY
        // =================================================

        let Some(guild_id) =
            msg.guild_id
        else {
            return;
        };

        let Ok(member) =
            guild_id
                .member(
                    &ctx.http,
                    msg.author.id,
                )
                .await
        else {
            return;
        };

        // =================================================
        // STAFF CHECK
        // =================================================

        if !Self::is_staff(&member) {

            Self::no_permission(
                &ctx,
                &msg,
            )
            .await;

            return;
        }

        // =================================================
        // !message <tekst>
        // =================================================

        if command == "message" {

            if args.is_empty() {

                let _ =
                    msg.channel_id
                        .say(
                            &ctx.http,
                            "❌ Użycie: \
                             `!message <tekst>`",
                        )
                        .await;

                return;
            }

            let _ =
                msg.channel_id
                    .say(
                        &ctx.http,
                        args,
                    )
                    .await;

            return;
        }
    }

    // =================================================
    // BUTTON INTERACTIONS
    // =================================================

    async fn interaction_create(
        &self,
        ctx: Context,
        interaction: Interaction,
    ) {

        let Interaction::Component(
            component
        ) = interaction
        else {
            return;
        };

        // =================================================
        // VERIFY BUTTON
        // =================================================

        if component.data.custom_id
            != VERIFY_BUTTON_ID
        {
            return;
        }

        // =================================================
        // GUILD
        // =================================================

        let Some(guild_id) =
            component.guild_id
        else {

            Self::ephemeral(
                &ctx,
                &component,
                "❌ Ta weryfikacja działa \
                 tylko na serwerze.",
            )
            .await;

            return;
        };

        // =================================================
        // ROLE
        // =================================================

        let roles =
            match guild_id
                .roles(&ctx.http)
                .await
            {
                Ok(roles) => roles,

                Err(error) => {

                    println!(
                        "❌ Nie udało się pobrać ról: {}",
                        error
                    );

                    Self::ephemeral(
                        &ctx,
                        &component,
                        "❌ Nie udało się \
                         pobrać ról serwera.",
                    )
                    .await;

                    return;
                }
            };

        // =================================================
        // MEMBER ROLE
        // =================================================

        let Some(member_role) =
            roles
                .values()
                .find(
                    |role| {
                        role.name
                            == MEMBER_ROLE_NAME
                    }
                )
        else {

            println!(
                "❌ Nie znaleziono roli `{}`.",
                MEMBER_ROLE_NAME
            );

            Self::ephemeral(
                &ctx,
                &component,
                "❌ Nie znaleziono \
                 roli `Member`.",
            )
            .await;

            return;
        };

        // =================================================
        // USER
        // =================================================

        let Ok(member) =
            guild_id
                .member(
                    &ctx.http,
                    component.user.id,
                )
                .await
        else {

            Self::ephemeral(
                &ctx,
                &component,
                "❌ Nie udało się \
                 pobrać twojego profilu.",
            )
            .await;

            return;
        };

        // =================================================
        // ALREADY VERIFIED
        // =================================================

        if member
            .roles
            .contains(&member_role.id)
        {

            Self::ephemeral(
                &ctx,
                &component,
                "ℹ️ Jesteś już zweryfikowany!",
            )
            .await;

            return;
        }

        // =================================================
        // ADD ROLE
        // =================================================

        match member
            .add_role(
                &ctx.http,
                member_role.id,
            )
            .await
        {

            Ok(_) => {

                Self::ephemeral(
                    &ctx,
                    &component,
                    "✅ Zostałeś zweryfikowany \
                     na ∞studio!",
                )
                .await;

                println!(
                    "✅ Zweryfikowano: {} ({})",
                    component.user.name,
                    component.user.id
                );
            }

            Err(error) => {

                println!(
                    "❌ Błąd nadawania roli: {}",
                    error
                );

                Self::ephemeral(
                    &ctx,
                    &component,
                    "❌ Bot nie może nadać \
                     roli `Member`.\n\n\
                     Sprawdź:\n\
                     • Manage Roles\n\
                     • hierarchię ról\n\
                     • Server Members Intent",
                )
                .await;
            }
        }
    }
}

// =====================================================
// MAIN
// =====================================================

#[tokio::main]
async fn main() {

    println!("=================================");
    println!("∞studio Bot");
    println!("Production Build");
    println!("=================================");

    // =================================================
    // TOKEN Z ENV
    // =================================================

    let token =
        match env::var("DISCORD_TOKEN") {

            Ok(token) => token,

            Err(_) => {

                println!(
                    "❌ Brakuje DISCORD_TOKEN!"
                );

                println!(
                    "Ustaw DISCORD_TOKEN \
                     w Render Environment Variables."
                );

                return;
            }
        };

    // =================================================
    // HEALTH SERVER
    // =================================================

    tokio::spawn(async {

        if let Err(error) =
            health_server().await
        {
            println!(
                "❌ Health server error: {}",
                error
            );
        }
    });

    // =================================================
    // DISCORD INTENTS
    // =================================================

    let intents =
        GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS;

    // =================================================
    // CLIENT
    // =================================================

    let mut client =
        match Client::builder(
            &token,
            intents,
        )
        .event_handler(Handler)
        .await
        {
            Ok(client) => client,

            Err(error) => {

                println!(
                    "❌ Nie udało się stworzyć \
                     klienta Discord: {}",
                    error
                );

                return;
            }
        };

    println!(
        "🚀 Uruchamianie ∞studio Bot..."
    );

    // =================================================
    // START DISCORD
    // =================================================

    if let Err(error) =
        client.start().await
    {
        println!(
            "❌ Discord client error: {}",
            error
        );
    }
}
