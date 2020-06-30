// use spookylock_sys::vt::VtStream;
// use tui::backend::{Backend, TermionBackend};
// // use termion::raw::RawTerminal;
// use std::io::Result;

// pub struct TermionSizedBackend<'vt>
// {
//     stream: VtStream<'vt>,
//     inner: TermionBackend<VtStream<'vt>>,
// }

// impl<'vt> TermionSizedBackend<'vt> {
//     pub fn new(stream: VtStream<'vt>) -> Result<Self> {
// 	use termion::raw::IntoRawMode;
// 	let stream2 = stream.try_clone()?;
// 	// let raw = stream.into_raw_mode()?;
// 	Ok(Self {
// 	    inner: TermionBackend::new(stream),
// 	    stream: stream2
// 	})
//     }
// }

// impl<'vt> Backend for TermionSizedBackend<'vt> {
//     fn draw<'a, I>(&mut self, content: I) -> Result<()>
// 	where I: Iterator<Item = (u16, u16, &'a tui::buffer::Cell)>
//     {
// 	self.inner.draw(content)
//     }

//     fn hide_cursor(&mut self) -> Result<()> {
// 	self.inner.hide_cursor()
//     }

//     fn show_cursor(&mut self) -> Result<()> {
// 	self.inner.show_cursor()
//     }

//     fn get_cursor(&mut self) -> Result<(u16, u16)> {
// 	self.inner.get_cursor()
//     }

//     fn set_cursor(&mut self, x: u16, y: u16) -> Result<()> {
// 	self.inner.set_cursor(x, y)
//     }

//     fn clear(&mut self) -> Result<()> {
// 	self.inner.clear()
//     }

//     fn size(&self) -> Result<tui::layout::Rect> {
// 	self.stream.size().map(|s| {
// 	    tui::layout::Rect::new(0, 0, s.cols, s.rows)
// 	})
//     }

//     fn flush(&mut self) -> Result<()> {
// 	self.inner.flush()
//     }
// }
