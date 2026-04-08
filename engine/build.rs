use std::path::Path;
use std::process::Command;

const FAUST_PREAMBLE: &str = r#"type FaustFloat = f32;
type F32 = f32;
type F64 = f64;

#[derive(Copy, Clone, Debug)]
pub struct ParamIndex(pub i32);

pub trait Meta {
    fn declare(&mut self, key: &str, value: &str);
}

pub trait UI<T> {
    fn open_tab_box(&mut self, label: &str);
    fn open_horizontal_box(&mut self, label: &str);
    fn open_vertical_box(&mut self, label: &str);
    fn close_box(&mut self);
    fn add_button(&mut self, label: &str, param: ParamIndex);
    fn add_check_button(&mut self, label: &str, param: ParamIndex);
    fn add_vertical_slider(&mut self, label: &str, param: ParamIndex, init: T, min: T, max: T, step: T);
    fn add_horizontal_slider(&mut self, label: &str, param: ParamIndex, init: T, min: T, max: T, step: T);
    fn add_num_entry(&mut self, label: &str, param: ParamIndex, init: T, min: T, max: T, step: T);
    fn add_horizontal_bargraph(&mut self, label: &str, param: ParamIndex, min: T, max: T);
    fn add_vertical_bargraph(&mut self, label: &str, param: ParamIndex, min: T, max: T);
    fn declare(&mut self, param: Option<ParamIndex>, key: &str, value: &str);
}

pub trait FaustDsp {
    type T;
    fn new() -> Self where Self: Sized;
    fn metadata(&self, m: &mut dyn Meta);
    fn get_sample_rate(&self) -> i32;
    fn get_num_inputs(&self) -> i32;
    fn get_num_outputs(&self) -> i32;
    fn class_init(sample_rate: i32) where Self: Sized;
    fn instance_reset_params(&mut self);
    fn instance_clear(&mut self);
    fn instance_constants(&mut self, sample_rate: i32);
    fn instance_init(&mut self, sample_rate: i32);
    fn init(&mut self, sample_rate: i32);
    fn build_user_interface(&self, ui_interface: &mut dyn UI<Self::T>);
    fn build_user_interface_static(ui_interface: &mut dyn UI<Self::T>) where Self: Sized;
    fn get_param(&self, param: ParamIndex) -> Option<Self::T>;
    fn set_param(&mut self, param: ParamIndex, value: Self::T);
    fn compute(&mut self, count: i32, inputs: &[&[Self::T]], outputs: &mut [&mut [Self::T]]);
}
"#;

struct DspFile {
    src: &'static str,
    class_name: &'static str,
    out_name: &'static str,
}

const DSP_FILES: &[DspFile] = &[
    DspFile { src: "dsp/synth.dsp", class_name: "FaustSynth", out_name: "faust_synth.rs" },
    DspFile { src: "dsp/piano.dsp", class_name: "FaustPiano", out_name: "faust_piano.rs" },
];

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    for dsp in DSP_FILES {
        println!("cargo::rerun-if-changed={}", dsp.src);

        let out_file = Path::new(&out_dir).join(dsp.out_name);

        let status = Command::new("faust")
            .args(["-lang", "rust", "-cn", dsp.class_name, "-o"])
            .arg(&out_file)
            .arg(dsp.src)
            .status()
            .expect("failed to run faust compiler");

        assert!(status.success(), "faust compilation failed for {}", dsp.src);

        let generated = std::fs::read_to_string(&out_file).unwrap();
        let wrapped = format!("{FAUST_PREAMBLE}\n{generated}");
        std::fs::write(&out_file, wrapped).unwrap();
    }
}
