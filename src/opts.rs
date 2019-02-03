use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "subkatsu")]
pub enum Opts {
    #[structopt(
        name = "train",
        about = "Creates a model using subtitle files as training data"
    )]
    Train(Train),

    #[structopt(name = "generate", about = "Generates text from a model file")]
    Generate(Generate),
}

#[derive(Debug, StructOpt)]
pub struct Train {
    #[structopt(
        long = "output",
        short = "o",
        help = "Output destination for the model file"
    )]
    pub output: String,

    #[structopt(
        long = "order",
        default_value = "1",
        help = "Order of the Markov model"
    )]
    pub order: usize,

    #[structopt(
        required = true,
        help = "Training files to use as input (.srt/.ssa/.ass)"
    )]
    pub input: Vec<String>,
}

#[derive(Debug, StructOpt)]
pub struct Generate {
    #[structopt(help = "A model file generated from the training phase")]
    pub model: String,

    #[structopt(
        short = "n",
        default_value = "10",
        help = "Number of chains to generate"
    )]
    pub count: u32,

    #[structopt(
        long = "start-token",
        alias = "start",
        help = "Generate chains starting with this token"
    )]
    pub start: Option<String>,

    #[structopt(
        long = "min-length",
        help = "Generated chains must have at least this many characters"
    )]
    pub min_length: Option<usize>,
}
