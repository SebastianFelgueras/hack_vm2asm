//CONSIDERACIONES CONTRACTUALES Y BUGS
//el compilador no evita hacer operaciones sobre un stack vacio
//el compilador no siempre chequea que sea válida la direccion del memory segment a utilizar

use std::{
    env,
    process::exit,
    fs,
    path,
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
    if argumentos.len() == 2{
        if argumentos[1].trim() == "-v"{
            verboso = true;
        }else{
            inv_args!();
        }
    }else{
        verboso = false;
    }
    let mut compiler = comandos::Compiler::new(verboso);
    let archive = path::PathBuf::from(&argumentos[0]);
    if let Err(valor) = compiler.parse(archive.clone()){
        match valor.compilation_error(){
            comandos::CompilationError::FileAccessing{file}=>{
                println!("The compiler was unable to access the file: {}",file.to_str().unwrap());
                exit(-3);
            }
            comandos::CompilationError::SintaxError{line}=>{
                println!("Sintax error at line {} in file {}",line,valor.file_str());
                exit(-4);
            }
            comandos::CompilationError::UnknownCommand{line}=>{
                println!("Unknown command at line {} in file {}",line,valor.file_str());
                exit(-5);
            }
            comandos::CompilationError::UnknownMemorySegment{line}=>{
                println!("Unknown memory segment at line {} in file {}",line,valor.file_str());
                exit(-6);
            }
        }
    }
    if let Err(_) = fs::write(archive.with_extension("asm"),compiler.compile()){
        println!("Error writing to the asm file!");
        exit(-9);
    }
}