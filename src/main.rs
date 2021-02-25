//CONSIDERACIONES CONTRACTUALES Y BUGS
//el compilador no evita hacer operaciones sobre un stack vacio
//el compilador no siempre chequea que sea válida la direccion del memory segment a utilizar
//el compilador asume que no se usa multithreading (de hecho no lo soporta), sino puede haber problemas al parsear funciones
//En algunos casos limite puede haber problemas con las labels ya que una vez que se declara una funcion no siempre es claro donde termina,
//que aparezca un return no quiere decir que termine, puede ser un return dentro de un condicional
use std::{
    env,
    process::exit,
    fs,
    path,
};
mod comandos;
macro_rules! inv_args {
    () => {
        println!("Invalid arguments. First argument should be the name of the file to translate and the second (and optional) is \"-v\" if you want the translated commands to be present in the .asm file as comments");
        exit(-1);  
    };
}
fn main() {
    let argumentos = env::args().skip(1).collect::<Vec<String>>();
    if argumentos.len() == 0 || argumentos.len() > 3{
        inv_args!();
    }
    let verboso = argumentos.contains(&"-v".to_string());
    if verboso{
        println!("Verbose output to file!")
    }
    let mut compiler = comandos::Compiler::new(verboso);
    let mut archive = path::PathBuf::from(&argumentos[0]);
    if argumentos.contains(&"-bo".to_string()){
        compiler.disable_booting_code();
        println!("Booting code disabled!");
    }
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
            /*comandos::CompilationError::UnknownLabel{line}=>{
                println!("Unknown label at line {} in file {}",line,valor.file_str());
                exit(-7);
            }*/
        }
    }
    archive.push(archive.file_name().unwrap().to_owned());
    if let Err(_) = fs::write(archive.with_extension("asm"),compiler.compile()){
        println!("Error writing to the asm file!");
        exit(-9);
    }
}