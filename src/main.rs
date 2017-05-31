#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate file;
extern crate rand;

use rocket_contrib::JSON;
use rocket::response::content::Content;
use rocket::http::ContentType;

use std::collections::HashSet;
use std::str::FromStr;

use rand::Rng;

#[derive(Serialize, Debug, Clone, Copy)]
enum Sex {
    Male,
    Female
}

impl rand::Rand for Sex {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        let variants = [Sex::Male, Sex::Female];
        *rng.choose(&variants).unwrap()
    }
}

#[derive(Serialize, Debug)]
struct Person {
    sex: Sex,
    first_name: String,
    second_name: String,
    last_name: String
}

#[derive(Debug)]
struct NamePool {
    female: Vec<String>,
    male: Vec<String>,
    surname: Vec<String>,
}

impl NamePool {
    fn random_name(&self) -> Person {
        fn random_value(vec: &Vec<String>) -> &String {
            // todo unwrap_or
            rand::thread_rng().choose(vec).unwrap()
        }

        let sex: Sex = rand::random();
        let first_name = match sex {
            Sex::Male => random_value(&self.male),
            Sex::Female => random_value(&self.female)
        };

        let person = Person {
            sex: sex,
            first_name: first_name.to_string(),
            second_name: random_value(&self.male).to_string(),
            last_name: random_value(&self.surname).to_string()
        };

        println!("{:?}", person);
        person
    }
}

#[get("/")]
fn index(pool: rocket::State<NamePool>) -> JSON<Person> {
    let person = pool.random_name();
    JSON(person)
}

fn json_workaround() -> ContentType {
    ContentType::from_str("application/json; encoding=utf-8").unwrap()
}

#[get("/2")]
fn index2(pool: rocket::State<NamePool>) -> Content<String> {
    let person = pool.random_name();
    let json = serde_json::to_string(&person).unwrap();
    Content(json_workaround(), json)
}

fn load_name_pool() -> NamePool {
    let female_names = merge_names(&[
        "src/names/female-names-v1-14376.txt",
        "src/names/female-names-v2-16673.txt",
    ]);

    let male_names = merge_names(&[
        "src/names/male-names-v1-14999.txt",
        "src/names/male-names-v2-21904.txt",
    ]);

    let surnames = merge_names(&[
        "src/names/surnames-184624.txt",
    ]);

    NamePool { female: female_names, male: male_names, surname: surnames }
}

fn merge_names(filenames: &[&str]) -> Vec<String> {
    let mut distinct = HashSet::new();
    for filename in filenames {
        let names = load_from_file(filename);
        for item in names {
            distinct.insert(item);
        }
    }

    let mut result = vec![];
    for item in distinct {
        result.push(item);
    }

    result
}

fn load_from_file(filename: &str) -> HashSet<String> {
    let content = Box::new(file::get_text(filename).unwrap_or("".to_string()));
    let mut names = HashSet::new();
    for line in content.as_ref().split_whitespace() {
        names.insert(line.to_string());
    }
    names
}

fn main() {
    let pool = load_name_pool();
    rocket::ignite()
        .manage(pool)
        .mount("/", routes![index, index2]).launch();
}
