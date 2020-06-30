use spookylock_sys::vt;
use termion::raw::IntoRawMode;
// use tui::style::{Color, Style};
// use tui::widgets::{Block, Borders, Widget};
// use tui::Terminal;
use cursive::Cursive;

fn main() -> std::io::Result<()> {
    let console = vt::Console::acquire()?;
    let target = console.new_vt();

    console.on_vt(&target, || {
        let stdout = std::io::stdout();
        let stream = target.stream()?;
	stream.set_as_controlling_tty()?;

	let _guard = stream_switch::Guard::acquire_write(&stream, &stdout)?;
	let _s = std::io::stdout().into_raw_mode()?;

	let mut siv = Cursive::termion()?;

	let mut theme = cursive::theme::Theme::default();
	let mut palette = cursive::theme::Palette::default();
	let b = palette[cursive::theme::PaletteColor::Background];
	let k = palette[cursive::theme::PaletteColor::Primary];

	palette[cursive::theme::PaletteColor::Background] = k;
	palette[cursive::theme::PaletteColor::Shadow] = b;
	palette[cursive::theme::PaletteColor::Primary] = b;
	palette[cursive::theme::PaletteColor::Secondary] = k;
	theme.palette = palette;

	siv.set_theme(theme);

	use cursive::view::{Boxable, Identifiable};

	let view = cursive::views::LinearLayout::vertical()
	    .child(cursive::views::DummyView)
	    .child(cursive::views::LinearLayout::horizontal()
		   .child(cursive::views::TextView::new("Password"))
		   .child(cursive::views::DummyView)
		   .child(cursive::views::EditView::new()
			  .secret()
			  .on_submit(|s, t| {
			      try_login(s, t);
			  })
			  .with_id("pw")
			  .fixed_width(32)));

	siv.add_layer(cursive::views::Dialog::around(view)
		      .title("Locked"));
	siv.add_global_callback(cursive::event::Event::CtrlChar('u'), |s: &mut Cursive| {
	    s.call_on_id("pw", |v: &mut cursive::views::EditView| {
		v.set_content("");
	    });
	});
	
	siv.run();
	
	Ok(())
    })
}

fn try_login(s: &mut Cursive, pw: &str) -> Result<(), pam::PamError> {
    let mut auth = pam::Authenticator::with_password("system-auth")?;
    auth.get_handler().set_credentials("toshokan", pw);
    if let Ok(_) = auth.authenticate() {
	s.quit()
    } else {
	s.call_on_id("pw", |v: &mut cursive::views::EditView| {
	    v.set_content("");
	});
    }
    Ok(())
}
