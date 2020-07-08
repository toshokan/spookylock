use clap::Clap;
use cursive::Cursive;

#[derive(Clap)]
struct Options {
    #[clap(short, long)]
    user: String,
}

fn try_login(s: &mut Cursive, user: &str, pw: &str) -> Result<(), pam::PamError> {
    let mut auth = pam::Authenticator::with_password("system-auth")?;
    auth.get_handler().set_credentials(user, pw);
    if let Ok(_) = auth.authenticate() {
        s.quit()
    } else {
        s.call_on_id("pw", |v: &mut cursive::views::EditView| {
            v.set_content("");
        });
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let opts = Options::parse();

    let mut siv = Cursive::termion()?;

    let mut theme = cursive::theme::Theme::default();
    let mut palette = cursive::theme::Palette::default();
    let b = palette[cursive::theme::PaletteColor::Background];
    let k = palette[cursive::theme::PaletteColor::Primary];

    palette[cursive::theme::PaletteColor::Background] = k;
    palette[cursive::theme::PaletteColor::Shadow] = k;
    palette[cursive::theme::PaletteColor::Primary] = b;
    palette[cursive::theme::PaletteColor::Secondary] = k;
    theme.palette = palette;

    siv.set_theme(theme);

    use cursive::view::{Boxable, Identifiable};

    let view = cursive::views::LinearLayout::vertical()
        .child(cursive::views::DummyView)
        .child(
            cursive::views::LinearLayout::horizontal()
                .child(cursive::views::TextView::new("Password"))
                .child(cursive::views::DummyView)
                .child(
                    cursive::views::EditView::new()
                        .secret()
                        .on_submit(move |s, t| {
                            let _ = try_login(s, &opts.user, t);
                        })
                        .with_id("pw")
                        .fixed_width(32),
                ),
        );

    siv.add_layer(cursive::views::Dialog::around(view).title("Locked"));
    siv.add_global_callback(cursive::event::Event::CtrlChar('u'), |s: &mut Cursive| {
        s.call_on_id("pw", |v: &mut cursive::views::EditView| {
            v.set_content("");
        });
    });

    siv.run();

    Ok(())
}
