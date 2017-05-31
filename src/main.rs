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

use patronymic::Sex;

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
    patronymic: String,
    surname: String
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
            patronymic: patronymic::from_name(random_value(&self.male), sex).to_string(),
            surname: random_value(&self.surname).to_string()
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

mod patronymic {
    fn is_vowel(letter: char) -> bool {
        match letter {
            'а' | 'е' | 'ё' | 'и' | 'о' | 'у' | 'ы' | 'э' | 'ю' | 'я' => true,
            _ => false
        }
    }

    #[derive(Serialize, Debug, Clone, Copy, PartialEq)]
    pub enum Sex {
        Male,
        Female
    }

    /// http://zags.kurganobl.ru/obrazovanie_i_napisanie_otchestv.html
    ///
    /// Отчества от мужских имен (русских и нерусских) в русском языке образуются по следующим правилам:
    ///
    /// 1. Если имя оканчивается на твёрдый согласный (кроме ж, ш, ч, щ, ц), добавляется -ович / овна:
    /// Александр + ович/овна, Иван + ович/овна, Гамзат + ович/овна.
    ///
    /// 2. К именам, оканчивающимся на ж, ш, ч, щ, ц, добавляется -евич / евна:
    /// Жорж + евич/евна, Януш + евич/евна, Милич + евич/евна, Франц + евич/евна.
    ///
    /// 3. Если имя оканчивается на неударный гласный а, у, ы, к нему добавляется -ович / овна, причём конечные гласные имени отбрасываются:
    /// Антипа - Антипович/Антиповна, Вавила - Вавилович/ Вавиловна.
    /// Исключение: от русских имён Аникита, Никита, Мина, Савва, Сила, Фока образуются традиционные формы отчеств на -ич / ична:
    /// Никита - Никитич/ Никитична, Мина - Минич/Минична, Савва — Саввич/Саввична.
    ///
    /// 4. Если имя оканчивается на неударный гласный о, к нему добавляется -ович / овна, причём конечный гласный имени и начальный суффикса сливаются в один звук "о":
    /// Василько + ович/овна, Михайло + ович/овна, Отто + ович/овна.
    ///
    /// 5. Если неударному конечному гласному предшествует ж или ш, ч, щ, ц, то добавляется -евич / евна, а гласный отбрасывается:
    /// Важа - Важевич/Важевна, Гоча - Гочевич/Гочевна.
    ///
    /// 6. Если имя оканчивается на мягкий согласный, т. е. на согласный + ь, к нему добавляется -евич / евна, а конечный ь отбрасывается:
    /// Игорь - Игоревич/Игоревна, Цезарь - Цезаревич/Цезаревна, Виль - Вилевич/Вилевна, Камиль — Камилевич/Камилевна.
    ///
    /// 7. Если имя оканчивается на неударный гласный е, к нему добавляется -евич / евна, причём конечный гласный имени и начальный суффикса сливаются:
    /// Аарне - Аарневич/Аарневна, Григоре - Григоревич/Григоревна, Вилье — Вильевич/Вильевна.
    ///
    /// 8. Если имя оканчивается на неударный гласный и, к нему добавляется -евич / евна, при этом конечный гласный сохраняется:
    /// Вилли - Виллиевич/Виллиевна, Илмари - Илмариевич/Илмариевна.
    ///
    /// 9. Если имя оканчивается на неударное сочетание ий, к нему добавляется -евич / евна, причём конечный й отбрасывается, а предпоследний и либо переходит в ь, либо остаётся:
    /// а) переходит в ь после одного согласного или группы нт:
    /// Василий - Васильевич/Васильевна, Марий - Марьевич/Марьевна, Юлий — Юльевич/Юльевна.
    /// б) остаётся после к, х, ц, а также после двух согласных (кроме группы нт ):
    /// Никий - Никиевич/Никиевна, Люций - Люциевич/ Люциевна, Стахий — Стахиевич/Стахиевна.
    ///
    /// 10. Старые русские имена, оканчивающиеся на сочетания ея и ия , образуют отчества прибавлением -евич / евна, при этом конечное я отбрасывается, а и или е сохраняется:
    /// Менея - Менеевич/ Менеевна, Захария — Захариевич/Захариевна.
    ///
    /// 11. К именам, оканчивающимся на ударные гласные а, я, е, э, и, ы, ё, о, у, ю, добавляется -евич / евна, при этом конечный гласный сохраняется:
    /// Айбу - Айбуевич/Айбуевна, Бадма - Бадмаевич/Бадмаевна, Бату - Бутуевич/Батуевна, Вали - Валиевич/Валиевна, Дакко - Даккоевич/Даккоевна, Исе — Исеевич/Исеевна.
    ///
    /// 12. Имена, оканчивающиеся на ударные сочетания ай, яй, ей, эй, ий, ый, ой, уй, юй, образуют отчества прибавлением -евич / евна, причём конечный й отбрасывается:
    /// Акбай - Акбаевич/Акбаевна, Кий - Киевич/Киевна, Матвей — Матвеевич/Матвеевна.
    ///
    /// 13. Имена, оканчивающиеся на два гласных аа, ау, еу, ээ, ии, уу сохраняют их, образуя отчества прибавлением -евич / евна:
    /// Бимбии - Бимбииевич/Бимбииевна, Бобоо-Бобооевич/Бобооевна,Бурбээ—Бурбээевич/Бурбээевна.
    ///
    /// По настоянию заявителей возможны образования отчеств, которые несколько не соотвествуют изложенным правилам, так как с течением времени правила образования отчеств изменяются.
    /// Например, отчества от татарских и некоторых других имён имеют тенденцию выравниваться по русским образцам и принимать формы Набиуллович (вместо Набиуллаевич),
    /// Хамзевич (вместо Хамзяевич), Янович (вместо Янисович), Мариевич (вместо Мариусович) и т. д.
    /// Если у отца именуемого двойное имя, отчество для него образуется от любого из двух имен по выбору заявителей.
    /// Например, если отец Игорь-Эдуард,а ребенка назвали Антон, в его документах пишется либо Антон Игоревич, либо Антон Эдуардович,
    /// либо Антон Игорь-Эдуардович, как родители сочтут более удобным.
    pub fn from_name(first_name: &str, sex: Sex) -> String {
        fn last_two_letters(first_name: &str) -> (char, char) {
            let mut name = first_name.to_owned();
            let last = name.pop().unwrap();
            let prev = name.pop().unwrap();
            (last, prev)
        }

        fn concat_ending(name: &str, sex: Sex, male_end: &str, female_end: &str) -> String {
            if sex == Sex::Male {
                name.to_owned() + male_end
            } else {
                name.to_owned() + female_end
            }
        }

        if first_name.len() >= 2 {
            let (last, prev) = last_two_letters(first_name);
            if is_vowel(last) && is_vowel(prev) {
                return concat_ending(first_name, sex, "евич", "евна");
                //                let ending = if sex == Sex::Male { "евич" } else { "евна" };
                //                return first_name.to_owned() + ending;
            }
        }

        let mut name = first_name.to_owned();
        let last_letter = name.pop().unwrap();

        if first_name.ends_with("аа")
            || first_name.ends_with("ау")
            || first_name.ends_with("еу")
            || first_name.ends_with("ээ")
            || first_name.ends_with("ии")
            || first_name.ends_with("уу") {}

        let result = match last_letter {
            'б' | 'в' | 'г' | 'д' | 'р' | 'н' | 'т' => {
                // todo что такое твёрдый согласный?
                let ending = if sex == Sex::Male { "ович" } else { "овна" };
                first_name.to_owned() + ending
            }
            'й' => {
                let ending = if sex == Sex::Male { "евич" } else { "евна" };
                name.to_owned() + ending
            }
            'ж' | 'ш' | 'ч' | 'щ' | 'ц' => {
                let ending = if sex == Sex::Male { "евич" } else { "евна" };
                first_name.to_owned() + ending
            }
            'а' | 'у' | 'ы' | 'о' => {
                match first_name {
                    "Аникита" | "Никита" | "Мина" | "Савва" | "Сила" | "Фока" => {
                        let ending = if sex == Sex::Male { "ич" } else { "ична" };
                        name.to_owned() + ending
                    }
                    _ => {
                        let ending = if sex == Sex::Male { "ович" } else { "овна" };
                        name.to_owned() + ending
                    }
                }
            }
            _ => name
        };

        result.to_owned()
    }

    #[test]
    fn test_from_name() {
        fn test_name(name: &str, male_name: &str, female_name: &str) {
            assert_eq!(male_name, from_name(name, Sex::Male));
            assert_eq!(female_name, from_name(name, Sex::Female));
        }

        // 1.
        test_name("Александр", "Александрович", "Александровна");
        test_name("Иван", "Иванович", "Ивановна");
        test_name("Гамзат", "Гамзатович", "Гамзатовна");
        // 2.
        test_name("Андрей", "Андреевич", "Андреевна");
        test_name("Жорж", "Жоржевич", "Жоржевна");
        test_name("Милич", "Миличевич", "Миличевна");
        test_name("Франц", "Францевич", "Францевна");
        test_name("Януш", "Янушевич", "Янушевна");
        // 3.
        test_name("Антипа", "Антипович", "Антиповна");
        test_name("Вавила", "Вавилович", "Вавиловна");
        test_name("Бату", "Батович", "Батовна");
        test_name("Метлы", "Метлович", "Метловна");
        test_name("Аникита", "Аникитич", "Аникитична");
        test_name("Никита", "Никитич", "Никитична");
        test_name("Мина", "Минич", "Минична");
        test_name("Савва", "Саввич", "Саввична");
        test_name("Сила", "Силич", "Силична");
        test_name("Фока", "Фокич", "Фокична");
        // 4.
        test_name("Василько", "Василькович", "Васильковна");
        test_name("Отто", "Оттович", "Оттовна");
        // 5. todo
        //        test_name("Важа", "Важевич", "Важевна");
        //        test_name("Гоча", "Гочевич", "Гочевна");
        // 6.
        // 7.
        // 8.
        // 9.
        // 10.
        // 11.
        // 12.
        // 13.
        test_name("Бимбии", "Бимбииевич", "Бимбииевна");
        test_name("Бобоо", "Бобооевич", "Бобооевна");
        test_name("Бурбээ", "Бурбээевич", "Бурбээевна");
        // 14. double name
    }
}