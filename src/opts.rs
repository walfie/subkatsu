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
        default_value = "2",
        help = "Order of the Markov model. Higher values cause the generated \
                text to more closely resemble the training set."
    )]
    pub order: usize,

    #[structopt(
        required = true,
        help = "List of training files to use as input \
                (should have extensions .srt/.ssa/.ass)"
    )]
    pub input: Vec<String>,
}

#[derive(Debug, StructOpt)]
pub struct Generate {
    #[structopt(help = "Path to a model file generated from the training phase")]
    pub model: String,

    #[structopt(
        short = "n",
        long = "count",
        default_value = "25",
        help = "Number of chains to generate"
    )]
    pub count: u32,

    #[structopt(
        long = "start-token",
        alias = "start",
        help = "Generate chains starting with this token. \
                Note that this will only work if the model was trained with order = 1."
    )]
    pub start: Option<String>,

    #[structopt(
        long = "min-length",
        help = "Ensure that generated chains have at least this many characters"
    )]
    pub min_length: Option<usize>,
}
