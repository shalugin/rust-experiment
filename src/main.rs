#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate file;
extern crate rocket;
extern crate rand;

use std::collections::HashSet;
use rand::distributions::{IndependentSample, Range};

#[derive(Debug)]
struct Person {
    sex: bool,
    first_name: String,
    second_name: String,
    last_name: String
}

impl std::fmt::Display for Person {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let sex_name = if self.sex { "male" } else { "female" };
        write!(f, "({}, {} {} {})", sex_name, self.first_name, self.second_name, self.last_name)
    }
}

#[derive(Debug)]
struct NamePool {
    female: Vec<String>,
    male: Vec<String>,
    surname: Vec<String>,
}

impl NamePool {
    fn random_name(&self) -> String {
        fn random_value(vec: &Vec<String>) -> &String {
            let between = Range::new(0, vec.len() - 1);
            let mut rng = rand::thread_rng();
            let index = between.ind_sample(&mut rng);
            &vec[index]
        }

        let sex: bool = rand::random();
        let first_name = if sex {
            random_value(&self.male)
        } else {
            random_value(&self.female)
        };

        let person = Person {
            sex: sex,
            first_name: first_name.to_string(),
            second_name: random_value(&self.male).to_string(),
            last_name: random_value(&self.surname).to_string()
        };

        return format!("{}", person);
    }
}

struct NameGen {
    name_pool: NamePool,
    female_range: Range<usize>,
    //    male_range: Range<usize>,
    //    surname_range: Range<usize>,
    rng: rand::ThreadRng
}

struct XPool<'a> {
    names: &'a Vec<String>,
    range: Range<usize>,
    rng: rand::ThreadRng,
}

#[get("/")]
fn index(pool: rocket::State<NamePool>) -> String {
    pool.random_name()
}

//fn x() -> XPool<'static> {
//    let female_names = &merge_names(&[
//        "src/names/female-names-v1-14376.txt",
//        "src/names/female-names-v2-16673.txt",
//    ]);
//    let len = female_names.len();
//
//    let vv = XPool {
//        names: female_names,
//        range: Range::new(0, len),
//        rng: rand::thread_rng()
//    };
//    //
//    //
//    //    let pool = &load_name_pool();
//    //
//    //    NameGen {
//    //        name_pool: *pool,
//    //        female_range: Range::new(0, pool.female.len()),
//    //        rng: rand::thread_rng()
//    //    }
//    vv
//}

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
//    x();
    let pool = load_name_pool();
    rocket::ignite()
        .manage(pool)
        .mount("/", routes![index]).launch();
}
