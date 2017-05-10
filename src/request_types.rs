#[derive(Serialize,Deserialize,PartialEq,Eq)]
pub struct PostCommand {
    pub pid: usize,
    pub command: String
}

#[derive(Serialize,Deserialize,PartialEq,Eq)]
pub struct PostDeclarations {
    pub pid: usize,
    pub declarations: Vec<String>
}
