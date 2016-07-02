#[derive(Serialize,Deserialize,Display,PartialEq,Eq)]
pub struct PostCommand {
    pub pid: usize,
    pub command: String
}

#[derive(Serialize,Deserialize,Display,PartialEq,Eq)]
pub struct PostDeclarations {
    pub pid: usize,
    pub declarations: Vec<String>
}
