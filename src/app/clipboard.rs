use crate::editor::cursor::Selection;
use crate::editor::Editor;

pub fn copy_selection(editor: &Editor, sel: Selection) {
    if !sel.is_empty() {
        let (s, e) = sel.range();
        let text = editor
            .buffer()
            .as_str()
            .chars()
            .skip(s)
            .take(e - s)
            .collect::<String>();
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(text);
        }
    }
}

pub fn cut_selection(editor: &mut Editor, sel: Selection) {
    if !sel.is_empty() {
        let (s, e) = sel.range();
        let text = editor
            .buffer()
            .as_str()
            .chars()
            .skip(s)
            .take(e - s)
            .collect::<String>();
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(text.clone());
        }
        editor.buffer_mut().remove(s, e - s);
        editor.cursor_mut().set_position(s);
        editor
            .history_mut()
            .push(crate::editor::operations::Operation::Delete { position: s, text });
    }
}

pub fn paste_at_cursor(editor: &mut Editor) {
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        if let Ok(text) = clipboard.get_text() {
            if let Some(sel) = editor.cursor().selection() {
                if !sel.is_empty() {
                    let (s, e) = sel.range();
                    let txt = editor
                        .buffer()
                        .as_str()
                        .chars()
                        .skip(s)
                        .take(e - s)
                        .collect::<String>();
                    editor.buffer_mut().remove(s, e - s);
                    editor
                        .history_mut()
                        .push(crate::editor::operations::Operation::Delete {
                            position: s,
                            text: txt,
                        });
                    editor.cursor_mut().set_position(s);
                }
            }

            let pos = editor.cursor().position();
            editor.buffer_mut().insert(pos, &text);
            editor.cursor_mut().set_position(pos + text.len());
            editor
                .history_mut()
                .push(crate::editor::operations::Operation::Insert {
                    position: pos,
                    text,
                });
        }
    }
}
