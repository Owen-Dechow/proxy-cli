/*

public use:
    debug::print(content: String);

results in like box:
    +--PROXY--------------------------------+
    |                                       |
    |  Content                              |
    |  lines                                |
    |                                       |
    +---------------------------------------+

*/

const BOX_WIDTH: usize = 75;
const BOX_TITLE: &str = " PROXY ";
const VERTICAL: char = '|';
const HORIZONTAL: char = '-';
const CORNER: char = '+';
const HORIZONTAL_PAD: usize = 3;
const FLEX_SPLIT_WINDOW: usize = 20;

fn get_top_bar() -> String {
    let l_pad = String::from(HORIZONTAL).repeat(HORIZONTAL_PAD);
    let n_horizontal = BOX_WIDTH - BOX_TITLE.len() - 2 - HORIZONTAL_PAD;
    let r_pad = String::from(HORIZONTAL).repeat(n_horizontal);
    let bar = format!("{}{}{}{}{}", CORNER, l_pad, BOX_TITLE, r_pad, CORNER);

    return bar;
}

fn get_bottom_bar() -> String {
    let n_horizontal = BOX_WIDTH - 2;
    let center_area = String::from(HORIZONTAL).repeat(n_horizontal);
    let bar = format!("{}{}{}", CORNER, center_area, CORNER);

    return bar;
}

fn get_blank_row() -> String {
    let n_horizontal = BOX_WIDTH - 2;
    let center_area = String::from(" ").repeat(n_horizontal);
    let row = format!("{}{}{}", VERTICAL, center_area, VERTICAL);

    return row;
}

fn get_row_from_valid_content(content: String) -> String {
    let l_pad = String::from(" ").repeat(HORIZONTAL_PAD);
    let n_horizontal = BOX_WIDTH - 2 - HORIZONTAL_PAD - content.len();
    let r_pad = String::from(" ").repeat(n_horizontal);
    let row = format!("{}{}{}{}{}", VERTICAL, l_pad, content, r_pad, VERTICAL);

    return row;
}

fn get_content_rows(content: String) -> String {
    // Split on \n
    let mut content_rows = content.trim().split("\n").collect::<Vec<&str>>();

    // Calculate the max line length
    let max_content_length = BOX_WIDTH - 2 - HORIZONTAL_PAD * 2;

    // Split lines over max line length
    let mut ptr = 0;
    while ptr < content_rows.len() {
        if content_rows[ptr].len() > max_content_length {
            // Remove line from list
            let item = content_rows.remove(ptr);

            // Attempt to find space to split line on
            let mut flex_split = 0;
            while flex_split < FLEX_SPLIT_WINDOW
                && &item[max_content_length - flex_split..max_content_length - flex_split + 1]
                    != " "
            {
                flex_split += 1;
            }
            let split_idx = match flex_split >= FLEX_SPLIT_WINDOW {
                true => max_content_length,
                false => max_content_length - flex_split,
            };

            // Readd the line to the the vector to be rechecked and split again
            let a = &item[..split_idx];
            let b = &item[split_idx..];
            content_rows.insert(ptr, b.trim());
            content_rows.insert(ptr, a.trim());
        } else {
            ptr += 1;
        }
    }

    let mut rows = Vec::<String>::new();
    for content_row in content_rows {
        let row = get_row_from_valid_content(content_row.to_string());
        rows.push(row);
    }

    return rows.join("\n");
}

pub fn print(content: String) {
    println!("\n{}", get_top_bar());
    println!("{}", get_blank_row());
    println!("{}", get_content_rows(content));
    println!("{}", get_blank_row());
    println!("{}\n", get_bottom_bar());
}
