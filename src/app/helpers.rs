use crate::editor::Editor;

pub fn apply_undo(editor: &mut Editor, op: crate::editor::operations::Operation) {
    use crate::editor::operations::Operation;
    match op {
        Operation::Insert { position, text } => {
            editor.buffer_mut().remove(position, text.len());
            editor.cursor_mut().set_position(position);
        }
        Operation::Delete { position, text } => {
            editor.buffer_mut().insert(position, &text);
            editor.cursor_mut().set_position(position + text.len());
        }
    }
}

pub fn apply_redo(editor: &mut Editor, op: crate::editor::operations::Operation) {
    use crate::editor::operations::Operation;
    match op {
        Operation::Insert { position, text } => {
            editor.buffer_mut().insert(position, &text);
            editor.cursor_mut().set_position(position + text.len());
        }
        Operation::Delete { position, text } => {
            editor.buffer_mut().remove(position, text.len());
            editor.cursor_mut().set_position(position);
        }
    }
}

pub fn delete_selection_or_char(editor: &mut Editor) {
    use crate::editor::operations::Operation;

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
            editor.history_mut().push(Operation::Delete {
                position: s,
                text: txt,
            });
            editor.cursor_mut().set_position(s);
            return;
        }
    }

    let pos = editor.cursor().position();
    let buf_len = editor.buffer().len_chars();
    if pos < buf_len {
        if let Some(ch) = editor.buffer().get_char(pos) {
            editor.buffer_mut().remove(pos, 1);
            editor.history_mut().push(Operation::Delete {
                position: pos,
                text: ch.to_string(),
            });
        }
    }
}
