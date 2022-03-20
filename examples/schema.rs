use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use polls::msg::{VotesResponse, WinnerResponse, OptionsResponse, QuestionResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use polls::state::Config;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(Config), &out_dir);
    export_schema(&schema_for!(VotesResponse), &out_dir);
    export_schema(&schema_for!(WinnerResponse), &out_dir);
    export_schema(&schema_for!(OptionsResponse), &out_dir);
    export_schema(&schema_for!(QuestionResponse), &out_dir);
}
