// TODO: define what kind of checks to do

static TODO: &str = r#"
* generate one big AST using imports, check imports
    * load file by file, cache by name etc
    * use cache if multiple of same imports
* transform:
    * maybe simplify VarDeclEqual into VarDecl and Equal 
        * if it is splitted into two, how type inference would work?
    * SSA form for easier type checking?
* check types
    * separate all imported types into active/passive
    * create table for fields and methods
    * check each method, remember returns (make sure around if-else)
    * check that bang is only OK for active
    * check that active can both bang and call itself
    * fields of other actors are not accessible 
    * spawn and new return exactly what they need to return
    * default new and spawn take all args one by one

* what to do with caller and do we really
    need to allow everybody to be able to call everyone
* what if we allow passing Methods (introduce new type)?
* maybe simple interface-like stuff? e.g. implement something like traits
    traits/typeclasses sounds like a best way

"#;
