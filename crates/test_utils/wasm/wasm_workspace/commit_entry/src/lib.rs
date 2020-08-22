use hdk3::prelude::*;

#[hdk_entry(id = "post", required_validations = 5)]
struct Post(String);

entry_defs![Post::entry_def()];

fn post() -> Post {
    Post("foo".into())
}

#[hdk_extern]
fn commit_entry(_: ()) -> ExternResult<HeaderHash> {
    Ok(commit_entry!(post())?)
}

#[hdk_extern]
fn get_entry(_: ()) -> ExternResult<GetOutput> {
    Ok(GetOutput::new(get!(entry_hash!(post())?)?))
}