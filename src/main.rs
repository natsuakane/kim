use arboard::Clipboard;
use crossterm::{
    cursor::{position, MoveTo},
    event::{self, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
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

fn read_file(filename: &str) -> Vec<String> {
    match fs::read_to_string(filename) {
        Ok(contents) => contents.lines().map(String::from).collect(),
        Err(_) => vec![format!("ファイルを読み込めませんでした:{}", filename)],
    }
}

fn write_file(filename: &str, buf: &Vec<String>) -> io::Result<()> {
    let mut file = File::create(filename)?; // ファイルを作成
    for line in buf {
        writeln!(file, "{}", line)?; // 各行を書き込み（改行付き）
    }
    Ok(())
}

fn main() -> crossterm::Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let mut filepath = PathBuf::from(env::current_dir().unwrap());
    filepath.push(filename);

    //(loop (!= i 100) [(set i (+ i 1)) (paint 10 i (* i 100) i i)])
    let mut lex: script::Lexer = script::Lexer::new(String::from(
        "(set i 0) (loop (< i 30) [(set i (+ i 1)) (paint 0 i (* i 8) 0 0)])",
    ));
    lex.lex();
    let mut parser = script::Parser::new(lex);
    let mut interpreter: script::Interpreter = match parser.program() {
        Ok(pro) => script::Interpreter::new(pro),
        Err(msg) => {
            eprintln!("Parsing Error: {}.", msg);
            return Ok(());
        }
    };

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
    let mut input_buffer: Text = read_file(filepath.to_str().unwrap()); // 入力された文字を保持するバッファ
    let mut mode = Mode::Normal;
    let mut current_num = 0;
    let mut clipboard = Clipboard::new().unwrap();
    let mut recorder = UndoRedo::new();
    let (_, height) = terminal::size().unwrap();
    let mut upper: usize = 0;

    loop {
        // ユーザーの入力を待つ
        if event::poll(std::time::Duration::from_millis(100))? {
            if let event::Event::Key(key_event) = event::read().unwrap() {
                match key_event.code {
                    KeyCode::Enter => {
                        // Enterキーが押された場合、新しい行に移動
                        mode = Mode::Insert;
                        let mut spaces = String::new();
                        for i in 0..input_buffer[cursor_pos.1 + upper].len() {
                            if input_buffer[cursor_pos.1 + upper].chars().nth(i).unwrap() != ' ' {
                                break;
                            }
                            spaces += " ";
                        }
                        input_buffer.insert(cursor_pos.1 + upper + 1, spaces.clone());
                        cursor_pos.1 += 1;
                        cursor_pos.0 = CURSOR_START_POS + spaces.len();
                    }
                    KeyCode::Esc => {
                        mode = Mode::Normal;
                    }
                    KeyCode::Tab => {
                        input_buffer[cursor_pos.1] += "    ";
                        cursor_pos.0 += 4;
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
                                if input_buffer.len() != 0
                                    && cursor_pos.1 + upper < input_buffer.len() - 1
                                {
                                    cursor_pos.1 += 1;
                                    if input_buffer[cursor_pos.1 + upper].len()
                                        < cursor_pos.0 - CURSOR_START_POS
                                    {
                                        cursor_pos.0 = input_buffer[cursor_pos.1 + upper].len()
                                            + CURSOR_START_POS;
                                    }
                                    if cursor_pos.1 == height as usize
                                        && input_buffer.len() >= cursor_pos.1 + upper
                                    {
                                        upper += 1;
                                        cursor_pos.1 -= 1;
                                    }
                                }
                            }
                            'k' => {
                                if cursor_pos.1 > 0 {
                                    cursor_pos.1 -= 1;
                                    if input_buffer[cursor_pos.1 + upper].len()
                                        < cursor_pos.0 - CURSOR_START_POS
                                    {
                                        cursor_pos.0 = input_buffer[cursor_pos.1 + upper].len()
                                            + CURSOR_START_POS;
                                    }
                                } else if upper > 0 {
                                    upper -= 1;
                                }
                            }
                            'l' => {
                                if cursor_pos.0
                                    < input_buffer[cursor_pos.1 + upper].len() + CURSOR_START_POS
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
                                let mut spaces = String::new();
                                for i in 0..input_buffer[cursor_pos.1 + upper].len() {
                                    if input_buffer[cursor_pos.1 + upper].chars().nth(i).unwrap()
                                        != ' '
                                    {
                                        break;
                                    }
                                    spaces += " ";
                                }
                                input_buffer.insert(cursor_pos.1 + upper + 1, spaces.clone());
                                cursor_pos.1 += 1;
                                cursor_pos.0 = CURSOR_START_POS + spaces.len();
                            }
                            // remove char
                            'x' => {
                                if cursor_pos.0 > CURSOR_START_POS
                                    && input_buffer[cursor_pos.1 + upper].len() != 0
                                {
                                    input_buffer[cursor_pos.1 + upper]
                                        .remove(cursor_pos.0 - CURSOR_START_POS - 1);
                                    cursor_pos.0 -= 1;
                                }
                            }
                            'X' => {
                                if cursor_pos.0
                                    < input_buffer[cursor_pos.1 + upper].len() + CURSOR_START_POS
                                {
                                    input_buffer[cursor_pos.1 + upper]
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
                                    if cursor_pos.1 + upper >= input_buffer.len() {
                                        break;
                                    }
                                    if i != 0 {
                                        str += "\n";
                                    }
                                    str += &input_buffer[cursor_pos.1 + upper];
                                    input_buffer.remove(cursor_pos.1 as usize + upper);
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
                                    if cursor_pos.1 + upper + i as usize >= input_buffer.len() {
                                        break;
                                    }
                                    if i != 0 {
                                        str += "\n";
                                    }
                                    str += &input_buffer[cursor_pos.1 + i as usize + upper];
                                }
                                clipboard.set_text(str.as_str()).unwrap();
                                current_num = 0;
                            }
                            // paste clipboard
                            'p' => {
                                let str = clipboard.get_text().unwrap();
                                let cols: Vec<&str> = str.split('\n').collect();
                                for i in 0..cols.len() {
                                    input_buffer
                                        .insert(cursor_pos.1 + upper + i, cols[i].to_string());
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
                            '$' => {
                                cursor_pos.0 = input_buffer[cursor_pos.1].len() + CURSOR_START_POS;
                            }
                            '^' => {
                                cursor_pos.0 = CURSOR_START_POS;
                            }
                            'g' => {
                                cursor_pos.1 = 0;
                            }
                            'G' => {
                                if current_num == 0 {
                                    cursor_pos.1 = input_buffer.len() - 1;
                                } else {
                                    if current_num as usize >= input_buffer.len() {
                                        current_num = input_buffer.len() as i32;
                                    }
                                    if current_num < 5 {
                                        upper = 0;
                                    } else {
                                        upper = current_num as usize - 5;
                                    }
                                    cursor_pos.1 = current_num as usize - upper - 1;
                                    current_num = 0;
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
                            input_buffer[cursor_pos.1 + upper]
                                .insert(cursor_pos.0 - CURSOR_START_POS, c);
                            cursor_pos.0 += 1; // カーソル位置を右に移動
                        }
                    },
                    _ => {}
                }
            }
        }

        // 入力された内容を表示
        stdout.execute(MoveTo(0, 0))?; // カーソルを先頭に戻す
        stdout.execute(terminal::Clear(ClearType::All))?; // 画面をクリア

        // バッファを行単位で描画
        for (line_number, line) in input_buffer.iter().enumerate() {
            if line_number < upper || line_number >= upper + height as usize {
                continue;
            }
            // 行ごとに表示
            execute!(
                stdout,
                SetForegroundColor(Color::DarkYellow),
                Print(format!("{:>5} ", line_number + 1))
            )
            .unwrap();
            execute!(
                stdout,
                SetForegroundColor(Color::Grey),
                Print(format!("{}\r\n", line))
            )
            .unwrap();
        }
        execute!(stdout, Print(format!("{:>5} ", input_buffer.len()))).unwrap();
        // カーソルの位置を調整
        if cursor_pos.1 >= input_buffer.len() {
            cursor_pos.1 = input_buffer.len() - 1;
        }

        /*
        スクリプト処理
        match interpreter.execute() {
            Ok(res) => {
                for com in res {
                    match com {
                        script::Command::Paint(x, y, col) => {
                            execute!(
                                stdout,
                                MoveTo(x as u16, y as u16), // カーソル位置へ移動
                                SetBackgroundColor(col),    // 背景色を青に
                                Print(" "),                 // 1文字分塗る
                                ResetColor                  // 色をリセット
                            )
                            .unwrap();
                        }
                    }
                }
            }
            Err(msg) => {
                eprintln!("Execution Error: {}.", msg);
                return Ok(());
            }
        }
        */

        // カーソルを現在の位置に移動
        stdout.execute(MoveTo(cursor_pos.0 as u16, cursor_pos.1 as u16))?;
        stdout.flush()?; // バッファの内容を画面に反映
    }

    // 終了処理
    terminal::disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen).unwrap();
    match write_file(filename, input_buffer.clone().as_ref()) {
        Ok(_) => Ok(()),
        Err(_) => {
            eprintln!("Could not write to file '{}'!", filename);
            Ok(())
        }
    }
}
