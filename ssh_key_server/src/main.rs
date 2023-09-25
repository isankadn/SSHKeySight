#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive; 
use rocket::State;
use rocket_contrib::json::Json;
use std::collections::HashMap;
use std::sync::Mutex;
use rocket_contrib::templates::Template;
use bcrypt::{verify};
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use serde_yaml;
use base64;
use rocket::http::{Cookie, Cookies};
use rocket::response::Redirect;


struct AuthenticatedUser(String);

#[derive(Debug, Deserialize)]
struct User {
    username: String,
    password_hash: String,
}

#[derive(Debug, Deserialize)]
struct Credentials {
    users: Vec<User>,
}

#[derive(Serialize, Deserialize, Clone)]
struct SSHKeyReport {
    vm_name: String,
    ip_address: Option<String>,
    keys: Vec<String>,
}

struct MaybeAuthenticatedUser(Option<AuthenticatedUser>);

type KeyStorage = Mutex<HashMap<String, SSHKeyReport>>;

#[derive(Serialize, Debug)]
struct DisplayKey {
    key: String,
    owner_info: Option<String>,
}

#[derive(Debug)]
struct CustomError(String);

#[derive(FromForm)]
struct LoginForm {
    username: String,
    password: String,
}

fn load_credentials_from_file(file_path: &str) -> Result<Credentials, String> {
    let content = std::fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    serde_yaml::from_str(&content).map_err(|e| e.to_string())
}

fn verify_password(username: &str, password: &str, credentials: &Credentials) -> bool {
    println!("Verifying password for user: {}", username);
    if let Some(user) = credentials.users.iter().find(|u| &u.username == username) {
        println!("Found user: {:?}", user);
        match verify(password, &user.password_hash) {
            Ok(result) => {
                if result {
                    println!("Password verification successful");
                    return true;
                } else {
                    println!("Password verification failed");
                    return false;
                }
            },
            Err(e) => {
                println!("Error during password verification: {}", e);
                return false;
            }
        }
    }
    println!("User not found, cannot verify password");
    false
}


impl<'a, 'r> FromRequest<'a, 'r> for MaybeAuthenticatedUser {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let credentials = match load_credentials_from_file("credentials.yaml") {
            Ok(c) => c,
            Err(e) => {
                println!("Error loading credentials in FromRequest: {}", e);
                return Outcome::Forward(());
            }
        };
        println!("Credentials: {:?}", credentials);
        // Check for "username" cookie first
        let mut cookies = request.cookies();
        if cookies.get_private("username").is_some() {
            return Outcome::Success(MaybeAuthenticatedUser(Some(AuthenticatedUser("User".to_string()))));
        }

        // If no cookie, check the "Authorization" header for Basic Authentication
        let auth_header = request.headers().get_one("Authorization");
        
        if let Some(auth) = auth_header {
            if auth.starts_with("Basic ") {
                let base64_encoded = &auth[6..];
                if let Ok(decoded) = base64::decode(base64_encoded) {
                    if let Ok(auth_str) = String::from_utf8(decoded) {
                        let parts: Vec<&str> = auth_str.splitn(2, ':').collect();
                        if parts.len() == 2 {
                            let username = parts[0];
                            let password = parts[1];
                            if verify_password(username, password, &credentials) {
                                println!("Login successful for user: {}", username);
                                return Outcome::Success(MaybeAuthenticatedUser(Some(AuthenticatedUser(username.to_string()))));
                            }else {
                                println!("Login failed for user: {}", username);                                
                            }
                        }
                    }
                }
            }
        }
        println!("No valid login found");
        // If neither cookie nor "Authorization" header is valid, return None
        Outcome::Success(MaybeAuthenticatedUser(None))
    }
}



#[post("/", data = "<report>")]
fn receive_keys(report: Json<SSHKeyReport>, storage: State<'_, KeyStorage>) -> &'static str {
    let mut db = storage.lock().expect("Failed to lock storage.");
    db.insert(report.vm_name.clone(), report.clone());
    "Received"
}

#[get("/login")]
fn login_page() -> Template {
    Template::render("login", ())
}

#[post("/login", data = "<login_form>")]
fn login_submit(login_form: rocket::request::Form<LoginForm>, _storage: State<'_, KeyStorage>, mut cookies: Cookies) -> Redirect {
    let credentials = match load_credentials_from_file("credentials.yaml") {
        Ok(c) => c,
        Err(e) => {
            println!("Error loading credentials: {}", e);
            return Redirect::to("/login");
        }
    };
    println!("Credentials: {:?}", login_form.username);
    println!("Credentials: {:?}", login_form.password);
    if verify_password(&login_form.username, &login_form.password, &credentials) {
        // If the username and password are valid, set a private cookie
        cookies.add_private(Cookie::new("username", login_form.username.clone()));
        Redirect::to("/")
    } else {
        // If the username and password are invalid, redirect back to the login page
        Redirect::to("/login")
    }
}


#[get("/")]
fn list_keys(user: MaybeAuthenticatedUser, storage: State<'_, KeyStorage>) -> Result<Template, Redirect> {
    if user.0.is_none() {
        // If the user is not authenticated, redirect them to the login page
        return Err(Redirect::to("/login"));
    }

    let db = storage.lock().expect("Failed to lock storage.");

    let processed_data: HashMap<String, (Option<String>, Vec<DisplayKey>)> = 
        db.iter().map(|(_, report)| {
            let processed_keys: Vec<DisplayKey> = report.keys.iter().map(|key| {
                let parts: Vec<&str> = key.split_whitespace().collect();
                DisplayKey {
                    key: key.clone(),
                    owner_info: parts.get(2).cloned().map(String::from),
                }
            }).collect();
            (report.vm_name.clone(), (report.ip_address.clone(), processed_keys))
        }).collect();

    Ok(Template::render("index", &processed_data))
}


#[catch(404)]
fn not_found() -> rocket::response::Redirect {
    rocket::response::Redirect::to("/login")
}


fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .attach(Template::fairing()) // Attach the template fairing
        .manage(Mutex::new(HashMap::<String, SSHKeyReport>::new()))
        .mount("/", routes![receive_keys, list_keys, login_page, login_submit])
        .register(catchers![not_found])
}

fn main() {
    rocket().launch();
}
