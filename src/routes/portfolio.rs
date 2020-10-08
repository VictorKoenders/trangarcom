use super::Header;
use actix_web::{get, Responder};

#[derive(askama::Template)]
#[template(path = "portfolio.html")]
pub struct PortfolioList {
    header: Header<'static>,
    entries: Vec<Portfolio>,
}

pub struct Portfolio {
    image_url: &'static str,
    title_link: Option<&'static str>,
    title: &'static str,
    body: &'static str,
}

#[get("/portfolio")]
async fn list() -> impl Responder {
    super::respond_html_template(PortfolioList {
        header: Header {
            title: "Trangar.com",
            url: "/portfolio",
        },
        entries: vec![
            Portfolio {
                image_url: "/static/logo_white_knight.svg",
                title_link: Some("https://ridder.com"),
                title: "Ridder Growing Solutions",
                body: r#"
Ridder Growing Solutions provides solutions to all challenges in the agricultural industry.
This includes Process Automation, Water Treatment and Management systems.

Trangar helps develop several products, including [Ridder Productive](https://ridder.com/ridder-productive), [Ridder iScan](https://ridder.com/ridder-productive-scanning-system) and [Ridder HortiMaX-Go!](https://ridder.com/ridder-hortimax-go).
"#,
            },
            Portfolio {
                image_url: "/static/CoffeeByBenjamin.svg",
                title_link: Some("https://benjamin.coffee"),
                title: "Coffee by Benjamin",
                body: r#"
Coffee by Benjamin is a young startup that aims to bring the freshest coffee to your kitchen.
They provide the tools to roast your own coffee at home and allow you to tweak it to exactly the flavor you like.
For this to succeed they had to develop a multi-platform solution ranging from an embedded device, phone apps an a website.
They approached Trangar to develop their Android application, one that matches their high expectations.

Check out the android app in the [Google play store](https://play.google.com/store/apps/details?id=co.awkward.benjamin&hl=en_AU), or their project at [benjamin.coffee](https://benjamin.coffee).
"#,
            },
            Portfolio {
                image_url: "/static/pixelflut_logo.svg",
                title_link: Some("https://campzone.nl"),
                title: "Pixelflut @ Campzone",
                body: r#"
Campzone is a massive outdoor gaming event in the Netherlands.
It involves gaming, hanging out with friends, making good food, driving around with remote controlled carts, experimenting with smelting metals, and much more.

One of those events is called [Pixelflut](https://hackaday.com/2020/08/01/playing-the-pixelflut/).
Everyone on this event can access a central server, and tell that server to update a single pixel on a giant screen.
This provides an interesting platform for people as an introduction for programming, but also as a competition to see who can take over more pixels on the screen, as well as show full images or sometimes even movies.

Trangar has developed one of the fastest Pixelflut servers out there, peaking at 28gbit of data, while remaining a constant memory and CPU footprint for several days.
To reach such speeds, we had to develop an application in one of the fastest ![](/static/rust-logo-blk.svg)[programming languages](https://www.rust-lang.org/) out there.
The best part of this entire project is that we got to open-source it!
You can find it on [github.com/victorkoenders/pixelflut](https://github.com/victorkoenders/pixelflut).
"#,
            },
            Portfolio {
                image_url: "/static/Discord-Logo-Black.png",
                title_link: None,
                title: "Social media integrations",
                body: r#"
Trangar has made integrations with several social media applications over the years.
This includes ![](/static/twitter.svg)[Twitter](https://twitter.com),
![](/static/Discord-Logo-Black.png)[Discord](https://discord.com/),
![](/static/Telegram-Logo.png) [Telegram](https://telegram.org/) and
[IRC](https://en.wikipedia.org/wiki/Internet_Relay_Chat).

Integrations are commonly made to reach out to your customers, fans, friends or coworkers.
Another common use case is to collect information or follow accounts on from one platform and import that data. 
"#,
            },
        ],
    })
}

mod filters {
    pub fn markdown(s: &str) -> Result<String, core::fmt::Error> {
        let parser = pulldown_cmark::Parser::new(s);
        let mut output = String::new();
        pulldown_cmark::html::push_html(&mut output, parser);
        Ok(output)
    }
}
