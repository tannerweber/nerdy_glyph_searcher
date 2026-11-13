include!("src/cli.rs");

fn main() -> std::io::Result<()> {
    let out_dir = std::path::PathBuf::from(
        std::env::var_os("OUT_DIR").ok_or(std::io::ErrorKind::NotFound)?,
    );

    let cmd = Args::command();

    {
        let man = clap_mangen::Man::new(cmd);
        let mut buffer: Vec<u8> = Vec::default();
        man.render(&mut buffer)?;
        std::fs::write(out_dir.join(EXE_NAME.to_string() + ".1"), buffer)?;
    }

    for shell in Shell::get_all() {
        let mut buffer: Vec<u8> = Vec::default();
        shell.write_completion(&mut buffer);
        std::fs::write(
            out_dir.join(EXE_NAME.to_string() + &shell.extension()),
            buffer,
        )?;
    }

    Ok(())
}
