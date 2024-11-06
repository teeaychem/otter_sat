use otter_lib::{config::Config, context::Context, types::gen::Report};

fn value_of(variable: &str, context: &Context) -> Option<bool> {
    let mut the_value = None;
    if context.valuation_string().contains(variable) {
        the_value = Some(true)
    }
    if context
        .valuation_string()
        .contains(format!("-{variable}").as_str())
    {
        the_value = Some(false)
    }
    the_value
}

fn main() {
    let config = Config {
        polarity_lean: 0.0, // Always choose to value a variable false
        ..Default::default()
    };

    let mut the_context: Context = Context::default_config(config);

    assert!(the_context.clause_from_string("-p q").is_ok());

    assert!(the_context.solve().is_ok());

    assert_eq!(the_context.report(), Report::Satisfiable);

    assert_eq!(value_of("p", &the_context), Some(false));
    assert_eq!(value_of("q", &the_context), Some(false));

    the_context.clear_decisions();

    assert!(the_context.clause_from_string("p").is_ok());

    assert_eq!(value_of("p", &the_context), Some(true));

    assert!(the_context.solve().is_ok());

    assert_eq!(value_of("q", &the_context), Some(true));
    assert_eq!(the_context.report(), Report::Satisfiable);

    let an_error = the_context.clause_from_string("-p -q");
    assert!(an_error.is_err());

    the_context.clear_decisions();

    assert!(the_context.solve().is_ok());

    assert_eq!(the_context.report(), Report::Satisfiable);
}
