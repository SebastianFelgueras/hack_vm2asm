//Las label tienen scope global por ahora
static mut CALLID:usize = 0;
use std::{
    path,
    fs,
    collections::HashMap,
};
pub struct Compiler{
    comandos: Vec<(String,ComandosParseados)>,
    verboso: bool,
    pub booting_code:String,
}
impl Compiler{
    #[inline]
    pub fn new(verboso:bool)->Self{
        Compiler{
            comandos: Vec::new(),
            verboso,
            booting_code: Compiler::booting_code(),
        }
    }
    #[inline]
    fn booting_code()->String{
        format!("@256\nD=A\n@SP\nM=D\n{}\n",
        PossibleCommands::parse_command(vec!["call","Sys.init","0"],0,&HashMap::new()).unwrap().to_asm("",0))
    }
    pub fn parse(&mut self,archive:path::PathBuf)->Result<(),CompError>{
        if archive.is_file(){
            self.comandos.push(
                (
                    archive.file_name().unwrap().to_str().unwrap().to_string(),
                    match ComandosParseados::parse_commands(
                        match fs::read_to_string(&archive){
                            Ok(valor)=>valor,
                            Err(_)=>return Err(CompError{
                                compilation_error: CompilationError::FileAccessing{file:archive.clone()},
                                file: archive,
                            }),
                        },
                        self.verboso 
                    ){
                        Ok(valor)=>valor,
                        Err(valor)=>return Err(CompError{
                            compilation_error: valor,
                            file: archive,
                        }),
                    }
                )
            );
        }else{
            for file in archive.read_dir().unwrap(){
                let file = file.unwrap().path();
                if file.is_file() && file.extension() == Some(std::ffi::OsStr::new("vm")){
                    self.comandos.push(
                        (
                            file.file_name().unwrap().to_str().unwrap().to_string(),
                            match ComandosParseados::parse_commands(
                                    match fs::read_to_string(&file){
                                        Ok(valor)=>valor,
                                        Err(_)=>return Err(CompError{
                                            compilation_error: CompilationError::FileAccessing{file:file.clone()},
                                            file,
                                        }),
                                    },
                                    self.verboso 
                                ){
                                    Ok(valor)=>valor,
                                    Err(valor)=>return Err(CompError{
                                        compilation_error: valor,
                                        file,
                                    }),
                                }
                        )
                    );
                }
            }
        }
        Ok(())
    }
    pub fn compile(self)->String{
        let mut compilado = String::from(self.booting_code);
        for (current_file,comandos) in self.comandos{
            let mut comandos_str: Vec<String> = Vec::new();
            if self.verboso{
                comandos_str = comandos.comandos_str.unwrap();
            }
            let mut comandos_str = comandos_str.iter();
            for i in 0..comandos.comandos.len(){
                if self.verboso{
                    compilado += &format!("//{}\n",comandos_str.next().unwrap());
                }
                compilado += &comandos.comandos[i].to_asm(&current_file,i);
                compilado.push('\n');
            }
        }
        compilado
    }
}
#[derive(Debug)]
pub enum CompilationError{
    FileAccessing{file: path::PathBuf},
    UnknownCommand{line:usize},
    SintaxError{line:usize},
    UnknownMemorySegment{line:usize},
    UnknownLabel{line:usize},
}
pub struct CompError{
    compilation_error: CompilationError,
    file: path::PathBuf,
}
impl CompError{
    #[inline]
    pub fn compilation_error(&self)->&CompilationError{
        &self.compilation_error
    }
    #[inline]
    pub fn file_str(&self)->&str{
        self.file.to_str().unwrap()
    }
}
enum PossibleCommands{
    Memory(MemoryCommand),
    Arithmetic(ArithmeticCommand),
    Branching(BranchingCommand),
    Functions(FunctionCommand),
}
impl PossibleCommands{
    fn parse_command(line:Vec<&str>,line_number:usize,labels_on_scope:&HashMap<String,String>)->Result<PossibleCommands,CompilationError>{
        match line[0]{
            "pop"=>{
                Ok(
                    PossibleCommands::Memory(
                        MemoryCommand::Pop(
                            PossibleCommands::memory_location(&line,line_number)?
                        )
                    )
                )
            },
            "push"=>{
                Ok(
                    PossibleCommands::Memory(
                        MemoryCommand::Push(
                            PossibleCommands::memory_location(&line,line_number)?
                        )
                    )
                )
            },
            "add"=>{
                Ok(
                    PossibleCommands::Arithmetic(
                        ArithmeticCommand::Add
                    )
                )
            },
            "sub"=>{
                Ok(
                    PossibleCommands::Arithmetic(
                        ArithmeticCommand::Sub
                    )
                )
            },
            "neg"=>{
                Ok(
                    PossibleCommands::Arithmetic(
                        ArithmeticCommand::Neg
                    )
                )
            },
            "eq"=>{
                Ok(
                    PossibleCommands::Arithmetic(
                        ArithmeticCommand::Eq
                    )
                )
            },
            "gt"=>{
                Ok(
                    PossibleCommands::Arithmetic(
                        ArithmeticCommand::Gt
                    )
                )
            },
            "lt"=>{
                Ok(
                    PossibleCommands::Arithmetic(
                        ArithmeticCommand::Lt
                    )
                )
            },
            "and"=>{
                Ok(
                    PossibleCommands::Arithmetic(
                        ArithmeticCommand::And
                    )
                )
            },
            "or"=>{
                Ok(
                    PossibleCommands::Arithmetic(
                        ArithmeticCommand::Or
                    )
                )
            },
            "not"=>{
                Ok(
                    PossibleCommands::Arithmetic(
                        ArithmeticCommand::Not
                    )
                )
            },
            "label"=>{
                let label = match line.get(1){
                    Some(valor)=>valor,
                    None=>return Err(CompilationError::SintaxError{line:line_number})
                };
                let label_mangled = match labels_on_scope.get(*label){
                    Some(v)=>v,
                    None=>return Err(CompilationError::UnknownLabel{line:line_number})
                };
                Ok(
                    PossibleCommands::Branching(
                        BranchingCommand::Label(
                            label_mangled.clone()
                        )
                    )
                )
            },
            "if-goto"=>{
                let label = *match line.get(1){
                    Some(valor)=>valor,
                    None=>return Err(CompilationError::SintaxError{line:line_number})
                };
                if let Some(mangled) = labels_on_scope.get(label){
                    Ok(
                        PossibleCommands::Branching(
                            BranchingCommand::IfGoto(
                                mangled.to_string()
                            )
                        )
                    )
                }else{
                    Err(CompilationError::UnknownLabel{line:line_number})
                }
            },
            "goto"=>{
                let label = *match line.get(1){
                    Some(valor)=>valor,
                    None=>return Err(CompilationError::SintaxError{line:line_number})
                };
                if let Some(mangled) = labels_on_scope.get(label){
                    Ok(
                        PossibleCommands::Branching(
                            BranchingCommand::Goto(
                                mangled.to_string()
                            )
                        )
                    )
                }else{
                    Err(CompilationError::UnknownLabel{line:line_number})
                }
            },
            "function"=>{
                let name = match line.get(1){
                    Some(value)=>value.to_string(),
                    None=>return Err(CompilationError::SintaxError{line:line_number})
                };
                Ok(
                    PossibleCommands::Functions(
                        FunctionCommand::Function{
                            name,
                            n_locals: match line.get(2){
                                Some(value)=>{
                                    match value.parse(){
                                        Ok(v)=>v,
                                        Err(_)=>return Err(CompilationError::SintaxError{line:line_number}),
                                    }
                                },
                                None=>return Err(CompilationError::SintaxError{line:line_number})
                            }
                        }
                    )
                )
            },
            "call"=>{
                Ok(
                    PossibleCommands::Functions(
                        FunctionCommand::Call{
                            name: match line.get(1){
                                Some(value)=>value.to_string(),
                                None=>return Err(CompilationError::SintaxError{line:line_number})
                            },
                            n_args: match line.get(2){
                                Some(value)=>{
                                    match value.parse(){
                                        Ok(v)=>v,
                                        Err(_)=>return Err(CompilationError::SintaxError{line:line_number}),
                                    }
                                },
                                None=>return Err(CompilationError::SintaxError{line:line_number})
                            }
                        }
                    )
                )
            },
            "return"=>{
                Ok(
                    PossibleCommands::Functions(
                        FunctionCommand::Return
                    )
                )
            }
            _=>Err(CompilationError::UnknownCommand{line:line_number}),
        }
    }
    fn memory_location(line: &Vec<&str>,line_number:usize)->Result<MemoryLocation,CompilationError>{
        if line.len() != 3{
            return Err(CompilationError::SintaxError{line:line_number});
        }
        let parse = |num:&str|->Result<i16,CompilationError>{
            match num.parse(){
                Ok(valor)=>Ok(valor),
                Err(_)=>Err(CompilationError::SintaxError{line:line_number}),
            }
                
        };
        match line[1]{
            "static"=>{
                Ok(
                    MemoryLocation::Static(
                        parse(line[2])?
                    )
                )
            },
            "constant"=>{
                Ok(
                    MemoryLocation::Constant(
                        parse(line[2])?
                    )
                )
            },
            "local"=>{
                Ok(
                    MemoryLocation::Local(
                        parse(line[2])?
                    )
                )
            },
            "argument"=>{
                Ok(
                    MemoryLocation::Argument(
                        parse(line[2])?
                    )
                )
            },
            "pointer"=>{
                Ok(
                    MemoryLocation::Pointer(
                        parse(line[2])?
                    )
                )
            },
            "temp"=>{
                Ok(
                    MemoryLocation::Temp(
                        parse(line[2])?
                    )
                )
            },
            "this"=>{
                Ok(
                    MemoryLocation::This(
                        parse(line[2])?
                    )
                )
            },
            "that"=>{
                Ok(
                    MemoryLocation::That(
                        parse(line[2])?
                    )
                )
            },
            _=>Err(CompilationError::UnknownMemorySegment{line:line_number}),
        }
    }
    fn to_asm(&self,file_name:&str,current_command:usize)->String{
        match self{
            PossibleCommands::Memory(inner)=>{
                inner.to_asm(file_name)
            },
            PossibleCommands::Arithmetic(inner)=>{
                inner.to_asm(current_command)
            },
            PossibleCommands::Branching(inner)=>{
                inner.to_asm()
            },
            PossibleCommands::Functions(inner)=> inner.to_asm(),
        }
    }
}
enum MemoryCommand{
    Pop(MemoryLocation),
    Push(MemoryLocation),
}
impl MemoryCommand{
    fn to_asm(&self,file_name:&str)->String{
        match self{
            MemoryCommand::Pop(value)=>{
                let pop_signature = "@SP\nM=M-1\nA=M\nD=M\n";
                match value{
                    MemoryLocation::Constant(_)=>{
                        panic!("Invalid operation, cannot pop to constant");
                    },
                    MemoryLocation::Temp(direccion)=>{
                        if *direccion > 7{
                            panic!("Compilation error");
                        }
                        return format!("{}@{}\nM=D\n",pop_signature,*direccion+5);
                    },
                    MemoryLocation::Static(numero)=>{
                        return format!("{}@{}.{}\nM=D\n",pop_signature,file_name,numero);
                    },
                    MemoryLocation::Pointer(value)=>{
                        if *value == 0{
                            return format!("{}@THIS\nM=D\n",pop_signature);
                        }else if *value == 1{
                            return format!("{}@THAT\nM=D\n",pop_signature);
                        }else{
                            panic!("Compilation error");
                        }
                    },
                    valor=>{
                        let numero;
                        let seccion;
                        match valor{
                            MemoryLocation::Local(num)=>{
                                numero = num;
                                seccion = "LCL";
                            },
                            MemoryLocation::This(num)=>{
                                numero = num;
                                seccion = "THIS";
                            },
                            MemoryLocation::That(num)=>{
                                numero = num;
                                seccion = "THAT";
                            },
                            MemoryLocation::Argument(num)=>{
                                numero = num;
                                seccion = "ARG";
                            },
                            _=>unreachable!(),
                        }
                        return format!("@{}\nD=A\n@{}\nD=D+M\n@SP\nM=M-1\nA=M\nA=M\nA=A+D\nD=D-A\nA=A+D\nD=-D\nM=D",numero,seccion);
                    }
                }
            },
            MemoryCommand::Push(value)=>{
                //una vez que el valor a pushear esta en D, agregar esto
                let push_signature = "@SP\nA=M\nM=D\n@SP\nM=M+1";
                match value{
                    MemoryLocation::Constant(constante)=>{
                        return format!("@{}\nD=A\n{}",constante,push_signature)
                    },
                    MemoryLocation::Temp(direccion)=>{
                        if *direccion > 7{
                            panic!("Compilation error");
                        }
                        return format!("@{}\nD=M\n{}",direccion+5,push_signature);
                    },
                    MemoryLocation::Static(numero)=>{
                        return format!("@{}.{}\nD=M\n{}",file_name,numero,push_signature);
                    },
                    MemoryLocation::Pointer(value)=>{
                        if *value == 0{
                            return format!("@THIS\nD=M\n{}",push_signature);
                        }else if *value == 1{
                            return format!("@THAT\nD=M\n{}",push_signature);
                        }else{
                            panic!("Compilation error");
                        }
                    },
                    valor=>{
                        let numero;
                        let seccion;
                        match valor{
                            MemoryLocation::Local(num)=>{
                                numero = num;
                                seccion = "LCL";
                            },
                            MemoryLocation::This(num)=>{
                                numero = num;
                                seccion = "THIS";
                            },
                            MemoryLocation::That(num)=>{
                                numero = num;
                                seccion = "THAT";
                            },
                            MemoryLocation::Argument(num)=>{
                                numero = num;
                                seccion = "ARG";
                            },
                            _=>unreachable!(),
                        }
                        return format!("@{}\nD=A\n@{}\nA=M+D\nD=M\n{}",numero,seccion,push_signature);
                    }             
                }
            }
        }
    }
}
enum MemoryLocation{
    Static(i16),
    Constant(i16),
    Local(i16),
    Temp(i16),
    Pointer(i16),
    Argument(i16),
    This(i16),
    That(i16),
}
enum ArithmeticCommand{
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,
}
impl ArithmeticCommand{
    fn to_asm(&self,current_command:usize)->String{
        let binarias = "@SP\nM=M-1\nA=M\nD=M\n@SP\nA=M-1\nM=M";
        let unarias = "@SP\nA=M-1\nM=";
        let condiciones = "@SP\nM=M-1\nA=M\nD=M\nA=A-1\nD=M-D\n@ETIQUETAINTERNAINICIAL*\nD;<\n@SP\nA=M-1\nM=0\n@ETIQUETAFINALSALIDA*\n0;JMP\n(ETIQUETAINTERNAINICIAL*)\n@SP\nA=M-1\nM=-1\n(ETIQUETAFINALSALIDA*)";
        match self{
            ArithmeticCommand::Add=>{
                format!("{}+D",binarias)
            },
            ArithmeticCommand::Sub=>{
                format!("{}-D",binarias)
            }
            ArithmeticCommand::And=>{
                format!("{}&D",binarias)
            },
            ArithmeticCommand::Or=>{
                format!("{}|D",binarias)
            },
            ArithmeticCommand::Neg=>{
                format!("{}-M",unarias)
            },
            ArithmeticCommand::Not=>{
                format!("{}!M",unarias)
            },
            ArithmeticCommand::Eq=>{
                condiciones.replace('*', &current_command.to_string()).replace('<', "JEQ")

            },
            ArithmeticCommand::Gt=>{
                condiciones.replace('*', &current_command.to_string()).replace('<', "JGT")
            },
            ArithmeticCommand::Lt=>{
                condiciones.replace('*', &current_command.to_string()).replace('<', "JLT")
            }
        }
    }
}
struct ComandosParseados{
    pub comandos: Vec<PossibleCommands>,
    pub comandos_str: Option<Vec<String>>,
}
enum BranchingCommand{
    Goto(String),
    IfGoto(String),
    Label(String),
}
impl BranchingCommand{
    fn to_asm(&self)->String{
        match self{
            BranchingCommand::Goto(label)=>{
                format!("@{}\n0;JMP",label)
            },
            BranchingCommand::IfGoto(label)=>{
                format!("@SP\nM=M-1\nA=M\nD=M\n@{}\nD;JNE",label)
            },
            BranchingCommand::Label(label)=>{
                format!("({})",label)
            }
        }
    }
    fn mangle(current_function:&str,label:&str)->String{
        format!("{}${}",current_function,label)
    }
}
enum FunctionCommand{
    Function{name:String,n_locals:usize},
    Call{name:String,n_args:usize},
    Return,
}
impl FunctionCommand{
    fn to_asm(&self)->String{
        match self{
            FunctionCommand::Function{name,n_locals}=>{
                let fmang = format!("MANGLED::::{}::::NaMe",name);
                format!("({funcName})\n@{NLocals}\nD=A\n({funcNameMangled}START)\n@{funcNameMangled}END\nD;JEQ\n@SP\nA=M\nM=0\n@SP\nM=M+1\nD=D-1\n@{funcNameMangled}START\n0;JMP\n({funcNameMangled}END)",
                funcName=name,NLocals=n_locals,funcNameMangled=fmang)
            },
            FunctionCommand::Call{name,n_args}=>{
                let callid = format!("{}:::CALLID:::{}",name,unsafe{CALLID});
                unsafe{CALLID +=1};
                format!("@{label}\nD=A\n@SP\nA=M\nM=D\n@SP\nM=M+1\n@LCL\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n@ARG\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n@THIS\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n@THAT\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n@{nmas5}\nD=A\n@SP\nD=M-D\n@ARG\nM=D\n@SP\nD=M\n@LCL\nM=D\n@{funcion_nombre}\n0;JMP\n({label})"
                ,label=callid,nmas5=n_args+5,funcion_nombre=name)
            },
            FunctionCommand::Return=>{
                format!("@LCL\nD=M\n@R13\nM=D\n@5\nD=D-A\n@R14\nM=D\n@SP\nA=M-1\nD=M\n@ARG\nA=M\nM=D\n@ARG\nD=M+1\n@SP\nM=D\n@R13\nM=M-1\nA=M\nD=M\n@THAT\nM=D\n@R13\nM=M-1\nA=M\nD=M\n@THIS\nM=D\n@R13\nM=M-1\nA=M\nD=M\n@ARG\nM=D\n@R13\nM=M-1\nA=M\nD=M\n@LCL\nM=D\n@R14\nA=M\n0;JMP")
            }
        }
        
    }
}
impl ComandosParseados{
    pub fn parse_commands(texto: String,verbose:bool)->Result<ComandosParseados,CompilationError>{
        let mut comandos = Vec::new();
        let mut comandos_str:Vec<String> = Vec::new();
        let mut line_number = 1;
        let mut labels_on_scope = ComandosParseados::label_parser(&texto)?;
        'outer:for line in texto.lines(){
            let line = match strip_command(line){
                Some(valor)=>valor,
                None=>{line_number+=1;continue 'outer},
            };
            if verbose{
                comandos_str.push(line.to_string());
            }
            let line = line.split_whitespace().collect::<Vec<&str>>();
            //Si contiene 0 elementos hay algo que no anda como debería, debe tener al menos 1
            comandos.push(PossibleCommands::parse_command(line,line_number,&mut labels_on_scope)?);
            line_number +=1;
        }
        Ok(
            ComandosParseados{
                comandos,
                comandos_str: match verbose{
                    true=>Some(comandos_str),
                    false=>None,
                },
            }
        )
    }
    ///Asume que el mangling de las labels hace que sean privadas
    fn label_parser(texto:&String)->Result<HashMap<String,String>,CompilationError>{
        let mut mapa = HashMap::new();
        let mut current_function = "";
        let mut i =1;
        for line in texto.lines(){
            if let Some(valor) = strip_command(line){
                let valor = valor.split_whitespace().collect::<Vec<&str>>();
                if valor[0] =="label"{
                    mapa.insert(valor[1].to_string(),BranchingCommand::mangle(&current_function, valor[1]));
                }else if valor[0] == "function"{
                    match valor.get(1){
                        Some(v)=>current_function = v,
                        None=>return Err(CompilationError::SintaxError{line:i})
                    }
                }else if valor[0] == "return"{
                    current_function = "";
                }
                i+=1;
            }
        }
        Ok(mapa)
    }
}
fn strip_command(mut line:&str)->Option<&str>{
    line = line.trim();
    if line.starts_with("//"){
        return None;
    }else{
        if let Some(valor) = line.find("//"){
            line = &line[0..valor];
            if line.is_empty(){
                return None;
            }else{
                return Some(line);
            }
        }else{
            if line.is_empty(){
                return None;
            }else{
                return Some(line);
            }
        }
    }
}