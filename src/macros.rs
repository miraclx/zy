macro_rules! print_block {
    (
        $($k:literal => $v:expr)+
    ) => {
        {
            let mut lines = vec![];
            let mut l = (0, 0);
            $(
                let (k,v) = ($k,format!("{}", $v));
                l = (l.0.max(k.len()), l.1.max(v.len()).min(30));
                lines.push((k, v.len() > 30, v));
            )+
            println!("┌{}┐", "─".repeat(l.0 + l.1 + 7));
            for (k, q, v) in lines {
                println!("│ - {0:1$} : {2:3$} {4}", k, l.0, v, l.1, if q { "" } else { "│" });
            }
            println!("└{}┘", "─".repeat(l.0 + l.1 + 7));
        }
    };
}
