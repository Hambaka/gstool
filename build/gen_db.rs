use std::{env, fs::File, io::BufWriter, io::Write, path::Path};

pub fn run() {
  let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen_db.rs");
  let mut file = BufWriter::new(File::create(&path).unwrap());
  gen_essential(&mut file);
  gen_item_db(&mut file);
  gen_jinn_db(&mut file);
  gen_icon_db(&mut file);
}

fn gen_essential(file: &mut BufWriter<File>) {
  // Language enum
  writeln!(
    file,
    r#"#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {{
  Japanese = 0,
  English = 1,
  ChineseTBS = 2,
  ChineseTLA= 3,
}}

impl Language {{
  pub const LANGUAGES: [Self; 4] = [Language::Japanese, Language::English, Language::ChineseTBS, Language::ChineseTLA];

  pub fn name(self) -> &'static str {{
    match self {{
      Language::Japanese => "日文",
      Language::English => "英文",
      Language::ChineseTBS => "1代新汉化",
      Language::ChineseTLA => "2代汉化",
    }}
  }}
}}

"#
  )
  .unwrap();

  // Struct
  writeln!(
    file,
    r"pub struct PreviewObject {{
  name: [&'static str; 4],
}}

impl PreviewObject {{
  pub fn name(&self, lang: Language) -> &'static str {{
    self.name[lang as usize]
  }}
}}
"
  )
  .unwrap();

  writeln!(file).unwrap();
}

fn gen_item_db(file: &mut BufWriter<File>) {
  let jp =
    serde_json::from_slice::<Vec<String>>(include_bytes!("../resources/json/item_name/jp.json"))
      .unwrap();

  let en =
    serde_json::from_slice::<Vec<String>>(include_bytes!("../resources/json/item_name/en.json"))
      .unwrap();

  let chs1 =
    serde_json::from_slice::<Vec<String>>(include_bytes!("../resources/json/item_name/chs1.json"))
      .unwrap();

  let chs2 =
    serde_json::from_slice::<Vec<String>>(include_bytes!("../resources/json/item_name/chs2.json"))
      .unwrap();

  let en_dyrati = serde_json::from_slice::<Vec<String>>(include_bytes!(
    "../resources/json/item_name/en_dyrati.json"
  ))
  .unwrap();

  writeln!(file, "pub const ITEM_NAMES: [PreviewObject; 460] = [").unwrap();

  for i in 0..460 {
    writeln!(
      file,
      "  PreviewObject {{ name: [\"{}\", \"{}\", \"{}\", \"{}\"] }},",
      jp[i], en[i], chs1[i], chs2[i]
    )
    .unwrap();
  }

  writeln!(file, "];\n").unwrap();

  writeln!(file, "pub const DYRATI_ITEM_NAMES: [&str; 460] = [").unwrap();
  for item_name in en_dyrati.iter().take(460) {
    writeln!(file, "  \"{item_name}\",").unwrap();
  }
  writeln!(file, "];\n").unwrap();
}

fn gen_jinn_db(file: &mut BufWriter<File>) {
  let jp =
    serde_json::from_slice::<Vec<String>>(include_bytes!("../resources/json/djinn_name/jp.json"))
      .unwrap();

  let en =
    serde_json::from_slice::<Vec<String>>(include_bytes!("../resources/json/djinn_name/en.json"))
      .unwrap();

  let chs1 =
    serde_json::from_slice::<Vec<String>>(include_bytes!("../resources/json/djinn_name/chs1.json"))
      .unwrap();

  let chs2 =
    serde_json::from_slice::<Vec<String>>(include_bytes!("../resources/json/djinn_name/chs2.json"))
      .unwrap();

  writeln!(
    file,
    "pub const ALL_DJINN_NAMES: [[PreviewObject; 7]; 4] = ["
  )
  .unwrap();

  // 与 Element::GAMEPLAY_ORDER 保持一致
  let gameplay_order = [0, 2, 3, 1];

  for order in &gameplay_order {
    writeln!(file, "  [").unwrap();
    for j in order * 7..(order + 1) * 7 {
      writeln!(
        file,
        "    PreviewObject {{ name: [\"{}\", \"{}\", \"{}\", \"{}\"] }},",
        jp[j], en[j], chs1[j], chs2[j]
      )
      .unwrap();
    }
    writeln!(file, "  ],").unwrap();
  }
  writeln!(file, "];\n").unwrap();
}

fn gen_icon_db(file: &mut BufWriter<File>) {
  let en =
    serde_json::from_slice::<Vec<String>>(include_bytes!("../resources/json/item_name/en.json"))
      .unwrap();

  // "Mythril Bag"
  let mut first_mythril_bag = None;
  // "Rusty Sword"
  let mut first_rusty_sword = None;
  // "Rusty Axe"
  let mut first_rusty_axe = None;
  // "Rusty Mace"
  let mut first_rusty_mace = None;
  // "Rusty Staff"
  let mut first_rusty_staff = None;

  writeln!(file, "pub fn icon_id(item_id: u16) -> usize {{").unwrap();
  writeln!(file, "  match item_id {{").unwrap();

  let mut counter = 0;
  for (i, item_name) in en.iter().enumerate().take(460) {
    match item_name.as_str() {
      "?" | "Signal Whistle" => {
        // writeln!(file, "    {i} => 361,").unwrap();
        continue;
      }
      // Real one is 224
      "Mythril Bag" => {
        if let Some(val) = first_mythril_bag {
          writeln!(file, "    {i} => {val},").unwrap();
        } else {
          first_mythril_bag = Some(counter);
          writeln!(file, "    {i} => {counter},").unwrap();
          counter += 1;
        }
      }
      "Rusty Sword" => {
        if let Some(val) = first_rusty_sword {
          writeln!(file, "    {i} => {val},").unwrap();
        } else {
          first_rusty_sword = Some(counter);
          writeln!(file, "    {i} => {counter},").unwrap();
          counter += 1;
        }
      }
      "Rusty Axe" => {
        if let Some(val) = first_rusty_axe {
          writeln!(file, "    {i} => {val},").unwrap();
        } else {
          first_rusty_axe = Some(counter);
          writeln!(file, "    {i} => {counter},").unwrap();
          counter += 1;
        }
      }
      "Rusty Mace" => {
        if let Some(val) = first_rusty_mace {
          writeln!(file, "    {i} => {val},").unwrap();
        } else {
          first_rusty_mace = Some(counter);
          writeln!(file, "    {i} => {counter},").unwrap();
          counter += 1;
        }
      }
      "Rusty Staff" => {
        if let Some(val) = first_rusty_staff {
          writeln!(file, "    {i} => {val},").unwrap();
        } else {
          first_rusty_staff = Some(counter);
          writeln!(file, "    {i} => {counter},").unwrap();
          counter += 1;
        }
      }
      _ => {
        writeln!(file, "    {i} => {counter},").unwrap();
        counter += 1;
      }
    }
  }

  writeln!(file, "    _ => 361,").unwrap();
  writeln!(file, "  }}\n}}").unwrap();
}
