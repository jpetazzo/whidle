use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tiny_http::{Header, Response, Server};
use wayrs_client::protocol::WlSeat;
use wayrs_client::{Connection, EventCtx, IoMode};
use wayrs_protocols::ext_idle_notify_v1::{
    ext_idle_notification_v1, ExtIdleNotificationV1, ExtIdleNotifierV1,
};

// Short timeout so "idled" fires quickly, giving fine-grained idle tracking:
// once it fires we know activity stopped roughly one timeout ago.
const IDLE_TIMEOUT: Duration = Duration::from_secs(1);

fn idle_seconds(idle_since: Option<Instant>) -> u64 {
    idle_since.map_or(0, |since| since.elapsed().as_secs())
}

fn main() {
    let idle_state = Arc::new(Mutex::new(None::<Instant>));

    let wayland_state = Arc::clone(&idle_state);
    std::thread::spawn(move || run_wayland(wayland_state));

    let port = std::env::var("PORT").unwrap_or_else(|_| "2323".to_string());
    let server = Server::http(format!("0.0.0.0:{port}")).expect("failed to bind HTTP server");
    println!("whidle listening on :{port}");

    let content_type: Header = "Content-Type: application/json".parse().unwrap();
    for request in server.incoming_requests() {
        let ip = request
            .remote_addr()
            .map(|a| a.ip().to_string())
            .unwrap_or_else(|| "-".to_string());
        let seconds = idle_seconds(*idle_state.lock().unwrap());
        let body = format!("{{\"idleseconds\": {seconds}}}");

        println!(
            "{ip} \"{} {} HTTP/{}\" 200 {} idleseconds={seconds}",
            request.method(),
            request.url(),
            request.http_version(),
            body.len(),
        );

        let response = Response::from_string(body).with_header(content_type.clone());
        let _ = request.respond(response);
    }
}

fn run_wayland(idle_state: Arc<Mutex<Option<Instant>>>) {
    let mut conn = Connection::<()>::connect().expect("failed to connect to Wayland display");
    conn.blocking_roundtrip().expect("initial roundtrip failed");

    let seat: WlSeat = conn
        .bind_singleton(1..=1)
        .expect("compositor has no wl_seat");
    let idle_notifier: ExtIdleNotifierV1 = conn.bind_singleton(1..=1).expect(
        "compositor doesn't support ext_idle_notifier_v1 (is sway up to date? needs sway >= 1.8)",
    );

    idle_notifier.get_idle_notification_with_cb(
        &mut conn,
        IDLE_TIMEOUT.as_millis() as u32,
        seat,
        move |ctx: EventCtx<(), ExtIdleNotificationV1>| {
            let mut idle_since = idle_state.lock().unwrap();
            match ctx.event {
                ext_idle_notification_v1::Event::Idled => {
                    *idle_since = Some(Instant::now() - IDLE_TIMEOUT);
                }
                ext_idle_notification_v1::Event::Resumed => {
                    *idle_since = None;
                }
                _ => {}
            }
        },
    );

    loop {
        conn.flush(IoMode::Blocking).expect("wayland flush failed");
        conn.recv_events(IoMode::Blocking)
            .expect("wayland recv failed");
        conn.dispatch_events(&mut ());
    }
}
