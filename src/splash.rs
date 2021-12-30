// splash screen for giving user a interface,
// to load config, init, anohter things before run your main code.

// can we works in windows 7?

#[warn(dead_code)]

enum SplashType {
    Loading,
}

// splash page for loading
struct Splash {

}

impl Splash {

}

// styles:
// 1. image as background and text
// 2. logo and text


#[cfg(test)]
mod tests {
    use web_view::*;

    const SIMPLE_HTML: &str = r#"
<!doctype html>
<html>
    <head>
        <style>
            body {
                background-color: green;
            }
        </style>
    </head>
    <body>
        <h1>Hello, World!</h1>
    </body>
</html>
"#;

    #[test]
    fn test_simple_webview() {
        web_view::builder()
            .title("Webview 测试")
            .content(Content::Html(SIMPLE_HTML))
            .size(320, 240)
            .frameless(true)
            .user_data(())
            .invoke_handler(|wb, arg| {
                todo!()
            })
            .run()
            .unwrap();
    }
}
