// Imports
use crate::RnAppWindow;
use adw::{prelude::*, subclass::prelude::*};
use gtk4::{Button, CompositeTemplate, ListBox, ListBoxRow, Widget, glib, glib::clone};

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/bookmarksbrowser.ui")]
    pub(crate) struct RnBookmarksBrowser {
        #[template_child]
        pub(crate) headerbar: TemplateChild<gtk4::HeaderBar>,
        #[template_child]
        pub(crate) title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub(crate) add_bookmark_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) bookmarks_list: TemplateChild<ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnBookmarksBrowser {
        const NAME: &'static str = "RnBookmarksBrowser";
        type Type = super::RnBookmarksBrowser;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnBookmarksBrowser {
        fn constructed(&self) {
            self.parent_constructed();

            self.add_bookmark_button.connect_clicked(clone!(
                #[weak(rename_to=bookmarks_browser)]
                self.obj(),
                move |_| {
                    let _ = bookmarks_browser.activate_action("win.add-bookmark", None);
                }
            ));

            self.bookmarks_list.connect_row_activated(clone!(
                #[weak(rename_to=bookmarks_browser)]
                self.obj(),
                move |_, row| {
                    if let Some(index) = row_index(row) {
                        let _ = bookmarks_browser.activate_action(
                            "win.goto-bookmark",
                            Some(&(index as i32).to_variant()),
                        );
                    }
                }
            ));
        }

        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RnBookmarksBrowser {}
}

glib::wrapper! {
    pub(crate) struct RnBookmarksBrowser(ObjectSubclass<imp::RnBookmarksBrowser>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

/// The row index, stored as usize in the row's data.
fn row_index(row: &ListBoxRow) -> Option<usize> {
    // SAFETY: the pointer is valid while the row exists and the data was set by us.
    unsafe { row.data::<usize>("bookmark-index").map(|p| *p.as_ptr()) }
}

impl RnBookmarksBrowser {
    /// Rebuild the bookmarks list from the bookmarks of the canvas of the active tab.
    pub(crate) fn refresh_bookmarks_list(&self) {
        let imp = self.imp();
        while let Some(child) = imp.bookmarks_list.first_child() {
            imp.bookmarks_list.remove(&child);
        }

        let Some(appwindow) = self.root().and_downcast::<RnAppWindow>() else {
            return;
        };
        let Some(canvas) = appwindow.active_tab_canvas() else {
            return;
        };

        let bookmarks = canvas.engine_ref().bookmarks.clone();

        for (index, bookmark) in bookmarks.iter().enumerate() {
            let row = ListBoxRow::builder()
                .activatable(true)
                .css_classes(["activatable"])
                .build();
            // SAFETY: the key is unique per row.
            unsafe { row.set_data("bookmark-index", index) };

            let jump_button = Button::builder()
                .label(bookmark.name.clone())
                .css_classes(["flat"])
                .hexpand(true)
                .tooltip_text(format!("({:.0}, {:.0})", bookmark.pos.x, bookmark.pos.y))
                .build();
            let rename_button = Button::builder()
                .icon_name("edit-symbolic")
                .css_classes(["flat"])
                .tooltip_text("Rename")
                .build();
            let delete_button = Button::builder()
                .icon_name("selection-trash-symbolic")
                .css_classes(["flat"])
                .tooltip_text("Delete")
                .build();

            let list_box = imp.bookmarks_list.get();
            jump_button.connect_clicked(clone!(
                #[weak]
                row,
                #[weak]
                list_box,
                move |_| {
                    if let Some(index) = row_index(&row) {
                        let _ = list_box.activate_action(
                            "win.goto-bookmark",
                            Some(&(index as i32).to_variant()),
                        );
                    }
                }
            ));
            rename_button.connect_clicked(clone!(
                #[weak(rename_to=bookmarks_browser)]
                self,
                #[weak]
                row,
                move |_| {
                    if let Some(index) = row_index(&row) {
                        bookmarks_browser.open_rename_dialog(index);
                    }
                }
            ));
            delete_button.connect_clicked(clone!(
                #[weak(rename_to=bookmarks_browser)]
                self,
                #[weak]
                row,
                move |_| {
                    if let Some(index) = row_index(&row) {
                        bookmarks_browser.remove_bookmark(index);
                    }
                }
            ));

            let row_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(6)
                .margin_start(12)
                .margin_end(12)
                .margin_top(6)
                .margin_bottom(6)
                .build();
            row_box.append(&jump_button);
            row_box.append(&rename_button);
            row_box.append(&delete_button);

            row.set_child(Some(&row_box));
            imp.bookmarks_list.append(&row);
        }
    }

    /// Open a rename dialog for the bookmark at the given index.
    fn open_rename_dialog(&self, index: usize) {
        let Some(appwindow) = self.root().and_downcast::<RnAppWindow>() else {
            return;
        };
        let Some(canvas) = appwindow.active_tab_canvas() else {
            return;
        };

        let Some(current_name) = canvas
            .engine_ref()
            .bookmarks
            .get(index)
            .map(|b| b.name.clone())
        else {
            return;
        };

        let entry_row = adw::EntryRow::builder()
            .title("Name")
            .text(&current_name)
            .build();

        let dialog = adw::AlertDialog::builder()
            .heading("Rename bookmark")
            .body(format!("Bookmark {index}"))
            .build();
        dialog.set_extra_child(Some(&entry_row));
        dialog.add_response("cancel", "Cancel");
        dialog.add_response("rename", "Rename");
        dialog.set_response_appearance("rename", adw::ResponseAppearance::Suggested);
        dialog.set_default_response(Some("rename"));
        dialog.set_close_response("cancel");

        dialog.connect_response(
            None,
            clone!(
                #[weak(rename_to=bookmarks_browser)]
                self,
                move |_, response| {
                    if response == "rename" {
                        let name = entry_row.text().trim().to_string();
                        if !name.is_empty() {
                            bookmarks_browser.rename_bookmark(index, name);
                        }
                    }
                }
            ),
        );
        dialog.present(Some(&appwindow));
    }

    fn remove_bookmark(&self, index: usize) {
        if let Some(appwindow) = self.root().and_downcast::<RnAppWindow>()
            && let Some(canvas) = appwindow.active_tab_canvas()
        {
            let widget_flags = canvas.engine_mut().remove_bookmark(index);
            appwindow.handle_widget_flags(widget_flags, &canvas);
        }
    }

    fn rename_bookmark(&self, index: usize, name: String) {
        if let Some(appwindow) = self.root().and_downcast::<RnAppWindow>()
            && let Some(canvas) = appwindow.active_tab_canvas()
        {
            let widget_flags = canvas.engine_mut().rename_bookmark(index, name);
            appwindow.handle_widget_flags(widget_flags, &canvas);
        }
    }
}
