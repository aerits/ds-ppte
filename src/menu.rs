use citro2d_sys::C3D_RenderTarget_tag;

struct Page {
    links: Vec<Box<dyn PageModule>>
}

struct PageID(usize);
impl PageModule for PageID {
    fn render(&self, render_target: *mut C3D_RenderTarget_tag, selected: bool) {
        todo!()
    }
}

trait PageModule {
    fn render(&self, render_target: *mut C3D_RenderTarget_tag, selected: bool);
}

struct Pages {
    current_page: PageID,
    cursor_position: usize,
    pages: Vec<Page>
}