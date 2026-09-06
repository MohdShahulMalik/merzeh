## Instructions For Writting Tests
 - NEVER USE `serde_json` TO CREATE JSON OBJECTS, ALWAYS USE THE STRUCTS THAT ARE DEFINED IN THE CODEBASE OR DEFINE STRUCTS WHEN THEY ARE NOT ALREADY DEFINED TO USE JSON DATA AND LET SERDE SERIALIZE OR DESERIALIZE IT.
 - For sending requests to the enpoints that are server functions. ALWAYS look closely to their "input" field at the `#[server(input = something)]`. Like PatchJson means it will accept a http Patch Request with a json body. you like to use post for every request.
 - Use multiple test cases wherever necessary with rstest or however it is more appropriate.
 - All tests should be run with --features ssr
 - While working on fixing a particular thing or writing a test then only run those tests that are relevant

 ## Intructions For Checking Code Correctness
  - NEVER use plain ```cargo check``` instead combine it with with features flag like ```cargo check --features ssr``` (when changing something in the backend) and ```cargo check --features csr``` (when changing something in the frontend) or ```cargo check --features ssr,csr``` (when changing something in both frontend and backend)

 ## Intructions For Code Edits (That Apply to Tests Editing and Writting As Well)
  - Don't do unnecessary code edits that are not asked to do. If you think that a particular edit is necessary but is not asked to do then just simple ask me if I want to make you that edit as if you combine the unnecessary edits with the ones that were asked to do then it's not possible for me to reject that edit and then I will end up with some cascaded unnecessary edit.
  - NEVER write builder patterns like this ```let response = client.post(&fetch_url).json(&fetch_params).send().await.expect("Failed to fetch");``` instead write them like this ```
  let response = client.post(&fetch_url)
        .json(&fetch_params)
        .send()
        .await
        .expect("failed to fetch");```
 - NEVER use types or functions like this `crate::models::user::User`, always use a type or a function after importing it at the top.

## Intructions When Asked A Question
 - DON't go and just start changing or writting code in the codebase, use every other tool that the write tool and just answer the question properly!

## THINGS TO NEVER DO
 - NEVER EVER TRY TO ACCESS THE .env FILE AT ALL!! not through the read tool or by bash like using cat command or by any other means.
