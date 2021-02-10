use std::{
    env,
    process::exit,
    fs,
    path::Path,
};
mod comandos;

const STACK_BASE_ADDRESS:usize = 256; //Posicion del comienzo del stack
macro_rules! inv_args {
    () => {
        println!("Invalid arguments. First argument should be the name of the file to translate and the second (and optional) is \"-v\" if you want the translated commands to be present in the .asm file as comments");
        exit(-1);  
    };
}
fn main() {
    let argumentos = env::args().skip(1).collect::<Vec<String>>();
    if argumentos.len() == 0 || argumentos.len() > 2{
        inv_args!();
    }
    let verboso;
    let archive = Path::new(&argumentos[0]);
    let archivo = match fs::read_to_string(archive){
        Ok(valor)=>valor,
        Err(_)=>{println!("Error accessing the file");exit(-2)},
    };
    if argumentos.len() == 2{
        if argumentos[1].trim() == "-v"{
            verboso = true;
        }else{
            inv_args!();
        }
    }else{
        verboso = false;
    }
    let comandos = match comandos::ComandosParseados::parse_commands(archivo,verboso){
        Ok(valor)=>valor,
        Err(_)=>{println!("Compilation error");exit(-6) },
    };
    let mut compilado = String::new();
    let mut i = 0;
    let nombre_archivo = archive.file_stem().unwrap().to_str().unwrap();
    for comando in comandos.comandos{
        if verboso{
            compilado.push_str(
                &format!("//{}\n",
                match comandos.comandos_str{
                    Some(ref valor)=>&valor[i],
                    None=>panic!(""),
                })
            );
        }
        compilado.push_str(
            &comando.to_asm(nombre_archivo)
        )
    }
    
    if let Err(_) = fs::write(archive.with_extension("asm"),compilado){
        println!("Error writing to the asm file!");
        exit(-9);
    }
}