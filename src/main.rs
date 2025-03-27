use arboard::Clipboard;
use crossterm::{
    cursor,
    event::{self, KeyCode},
    execute,
    style::{Color, Print, SetForegroundColor},
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::collections::VecDeque;
use std::io::{self, Write};
mod script;

type Text = Vec<String>;

const MAX_UNDO: usize = 100;

struct UndoRedo {
    undo_stack: VecDeque<Text>,
    redo_stack: Vec<Text>,
}

impl UndoRedo {
    fn new() -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: Vec::new(),
        }
    }

    fn perform_action(&mut self, action: Text) {
        if self.undo_stack.len() == MAX_UNDO {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(action);
        self.redo_stack.clear();
    }

    fn undo(&mut self) -> Option<Text> {
        if let Some(last_action) = self.undo_stack.pop_back() {
            self.redo_stack.push(last_action.clone());
            return Some(last_action);
        }
        None
    }

    fn redo(&mut self) -> Option<Text> {
        if let Some(last_redo) = self.redo_stack.pop() {
            self.undo_stack.push_back(last_redo.clone());
            return Some(last_redo);
        }
        None
    }
}

enum Mode {
    Normal,
    Insert,
}

fn is_identifier_char(c: char) -> bool {
    match c {
        'a'..='z' => true,
        '_' => true,
        _ => false,
    }
}

fn main() -> crossterm::Result<()> {
    let mut result: String = String::new();
    let mut lex: script::Lexer = script::Lexer::new(String::from(
        "(set f (func {a b} (* (+ a 5) b))) (set g (func {f x} (f x 2))) (g f 5)",
    ));
    lex.lex();
    let parser = script::Parser::new(lex);
    match script::Interpreter::new(parser).execute() {
        Ok(res) => {
            for r in res {
                result += " ";
                result += &(r.clone());
            }
        }
        Err(msg) => {
            result += " ";
            result += &msg;
        }
    }

    // ターミナルの初期化
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?; // 生モード（キー入力をそのまま取得）
    stdout.execute(terminal::Clear(ClearType::All))?; // 画面をクリア
    execute!(
        stdout,
        EnterAlternateScreen,
        terminal::Clear(ClearType::All)
    )
    .unwrap();

    const CURSOR_START_POS: usize = 6;
    let mut cursor_pos = (CURSOR_START_POS, 0); // カーソルの初期位置
    let mut input_buffer: Text = vec![result]; // 入力された文字を保持するバッファ
    let mut mode = Mode::Normal;
    let mut current_num = 0;
    let mut clipboard = Clipboard::new().unwrap();
    let mut recorder = UndoRedo::new();

    loop {
        // ユーザーの入力を待つ
        if event::poll(std::time::Duration::from_millis(100))? {
            if let event::Event::Key(key_event) = event::read().unwrap() {
                match key_event.code {
                    KeyCode::Enter => {
                        // Enterキーが押された場合、新しい行に移動
                        input_buffer.insert(cursor_pos.1 + 1, String::new());
                        input_buffer[cursor_pos.1 + 1] = String::from(
                            input_buffer[cursor_pos.1]
                                .get(
                                    cursor_pos.0 - CURSOR_START_POS
                                        ..input_buffer[cursor_pos.1].len(),
                                )
                                .unwrap(),
                        );
                        input_buffer[cursor_pos.1] = String::from(
                            input_buffer[cursor_pos.1]
                                .get(0..cursor_pos.0 - CURSOR_START_POS)
                                .unwrap(),
                        );
                        cursor_pos.1 += 1; // 行番号をインクリメント
                        cursor_pos.0 = CURSOR_START_POS; // 行の先頭に戻す
                    }
                    KeyCode::Esc => {
                        mode = Mode::Normal;
                    }
                    KeyCode::Char(c) => match mode {
                        Mode::Normal => match c {
                            // manage numeric
                            '0'..='9' => {
                                current_num = current_num * 10 + (c as i32 - '0' as i32);
                            }
                            // redo undo
                            'u' => {
                                if let Some(data) = recorder.undo() {
                                    input_buffer = data;
                                }
                            }
                            'r' => {
                                if let Some(data) = recorder.redo() {
                                    input_buffer = data;
                                }
                            }
                            // move cursor
                            'h' => {
                                if cursor_pos.0 > CURSOR_START_POS {
                                    cursor_pos.0 -= 1;
                                }
                            }
                            'j' => {
                                if input_buffer.len() != 0 && cursor_pos.1 < input_buffer.len() - 1
                                {
                                    cursor_pos.1 += 1;
                                    if input_buffer[cursor_pos.1].len()
                                        < cursor_pos.0 - CURSOR_START_POS
                                    {
                                        cursor_pos.0 =
                                            input_buffer[cursor_pos.1].len() + CURSOR_START_POS;
                                    }
                                }
                            }
                            'k' => {
                                if cursor_pos.1 > 0 {
                                    cursor_pos.1 -= 1;
                                    if input_buffer[cursor_pos.1].len()
                                        < cursor_pos.0 - CURSOR_START_POS
                                    {
                                        cursor_pos.0 =
                                            input_buffer[cursor_pos.1].len() + CURSOR_START_POS;
                                    }
                                }
                            }
                            'l' => {
                                if cursor_pos.0
                                    < input_buffer[cursor_pos.1].len() + CURSOR_START_POS
                                {
                                    cursor_pos.0 += 1;
                                }
                            }
                            // quit
                            'q' => {
                                break;
                            }
                            // change mode to insert
                            'i' => {
                                mode = Mode::Insert;
                            }
                            'o' => {
                                mode = Mode::Insert;
                                input_buffer.insert(cursor_pos.1 + 1, String::new());
                                cursor_pos.1 += 1;
                                cursor_pos.0 = CURSOR_START_POS;
                            }
                            // remove char
                            'x' => {
                                if cursor_pos.0 > CURSOR_START_POS
                                    && input_buffer[cursor_pos.1].len() != 0
                                {
                                    input_buffer[cursor_pos.1]
                                        .remove(cursor_pos.0 - CURSOR_START_POS - 1);
                                    cursor_pos.0 -= 1;
                                }
                            }
                            'z' => {
                                if cursor_pos.0
                                    < input_buffer[cursor_pos.1].len() + CURSOR_START_POS
                                {
                                    input_buffer[cursor_pos.1]
                                        .remove(cursor_pos.0 - CURSOR_START_POS);
                                }
                            }
                            // remove and copy to clipboard
                            'd' => {
                                let mut str = String::new();
                                if current_num == 0 {
                                    current_num = 1;
                                }
                                for i in 0..current_num {
                                    if cursor_pos.1 >= input_buffer.len() {
                                        break;
                                    }
                                    if i != 0 {
                                        str += "\n";
                                    }
                                    str += &input_buffer[cursor_pos.1];
                                    input_buffer.remove(cursor_pos.1 as usize);
                                }
                                clipboard.set_text(str.as_str()).unwrap();
                                current_num = 0;
                                if input_buffer.len() == 0 {
                                    input_buffer.push(String::new());
                                    cursor_pos.0 = CURSOR_START_POS;
                                }
                            }
                            // write to clipboard
                            'y' => {
                                let mut str = String::new();
                                if current_num == 0 {
                                    current_num = 1;
                                }
                                for i in 0..current_num {
                                    if cursor_pos.1 + i as usize >= input_buffer.len() {
                                        break;
                                    }
                                    if i != 0 {
                                        str += "\n";
                                    }
                                    str += &input_buffer[cursor_pos.1 + i as usize];
                                }
                                clipboard.set_text(str.as_str()).unwrap();
                                current_num = 0;
                            }
                            // paste clipboard
                            'p' => {
                                let str = clipboard.get_text().unwrap();
                                let cols: Vec<&str> = str.split('\n').collect();
                                for i in 0..cols.len() {
                                    input_buffer.insert(cursor_pos.1 + i, cols[i].to_string());
                                }
                            }
                            // next or prev word
                            'w' => {
                                for i in (cursor_pos.0 - CURSOR_START_POS)
                                    ..input_buffer[cursor_pos.1].len()
                                {
                                    if is_identifier_char(
                                        input_buffer[cursor_pos.1].chars().nth(i).unwrap(),
                                    ) {
                                        cursor_pos.0 = i + CURSOR_START_POS
                                    } else {
                                        cursor_pos.0 += 1;
                                        break;
                                    }
                                }
                                for i in (cursor_pos.0 - CURSOR_START_POS)
                                    ..input_buffer[cursor_pos.1].len()
                                {
                                    if !is_identifier_char(
                                        input_buffer[cursor_pos.1].chars().nth(i).unwrap(),
                                    ) {
                                        cursor_pos.0 = i + CURSOR_START_POS;
                                    } else {
                                        cursor_pos.0 += 1;
                                        break;
                                    }
                                }
                            }
                            'b' => {
                                for i in (0..(cursor_pos.0 - CURSOR_START_POS)).rev() {
                                    if is_identifier_char(
                                        input_buffer[cursor_pos.1].chars().nth(i).unwrap(),
                                    ) {
                                        cursor_pos.0 = i + CURSOR_START_POS;
                                    } else {
                                        cursor_pos.0 -= 1;
                                        break;
                                    }
                                }
                                for i in (0..(cursor_pos.0 - CURSOR_START_POS)).rev() {
                                    if !is_identifier_char(
                                        input_buffer[cursor_pos.1].chars().nth(i).unwrap(),
                                    ) {
                                        cursor_pos.0 = i + CURSOR_START_POS;
                                    } else {
                                        cursor_pos.0 -= 1;
                                        break;
                                    }
                                }
                                for i in (0..(cursor_pos.0 - CURSOR_START_POS)).rev() {
                                    if is_identifier_char(
                                        input_buffer[cursor_pos.1].chars().nth(i).unwrap(),
                                    ) {
                                        cursor_pos.0 = i + CURSOR_START_POS;
                                    } else {
                                        break;
                                    }
                                }
                            }
                            _ => {}
                        },
                        Mode::Insert => {
                            match c {
                                '`' => {
                                    mode = Mode::Normal;
                                    continue;
                                }
                                ' ' => {
                                    recorder.perform_action(input_buffer.clone());
                                }
                                _ => {}
                            }
                            // 文字が入力された場合、それをバッファに追加
                            input_buffer[cursor_pos.1].insert(cursor_pos.0 - CURSOR_START_POS, c);
                            cursor_pos.0 += 1; // カーソル位置を右に移動
                        }
                    },
                    _ => {}
                }
            }
        }

        // 入力された内容を表示
        stdout.execute(cursor::MoveTo(0, 0))?; // カーソルを先頭に戻す
        stdout.execute(terminal::Clear(ClearType::All))?; // 画面をクリア

        // バッファを行単位で描画
        let mut cols = 0;
        for (line_number, line) in input_buffer.iter().enumerate() {
            // 行ごとに表示
            execute!(
                stdout,
                SetForegroundColor(Color::DarkYellow),
                Print(format!("{:>5} ", line_number))
            )
            .unwrap();
            execute!(
                stdout,
                SetForegroundColor(Color::Grey),
                Print(format!("{}\r\n", line))
            )
            .unwrap();
            cols += 1;
        }
        execute!(stdout, Print(format!("{:>5} ", cols))).unwrap();
        // カーソルの位置を調整
        if cursor_pos.1 >= input_buffer.len() {
            cursor_pos.1 = input_buffer.len() - 1;
        }
        if cursor_pos.0 > input_buffer.last().unwrap().len() + CURSOR_START_POS {
            cursor_pos.0 = input_buffer.last().unwrap().len() + CURSOR_START_POS - 1;
        }
        // カーソルを現在の位置に移動
        stdout.execute(cursor::MoveTo(cursor_pos.0 as u16, cursor_pos.1 as u16))?;
        stdout.flush()?; // バッファの内容を画面に反映
    }

    // 終了処理
    terminal::disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen).unwrap();
    Ok(())
}
