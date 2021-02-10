pub enum PossibleCommands{
    Memory(MemoryCommand),
    Arithmetic(ArithmeticCommand),
}
impl PossibleCommands{
    fn parse_command(line:Vec<&str>)->Result<PossibleCommands,()>{
        match line[0]{
            "pop"=>{
                Ok(
                    PossibleCommands::Memory(
                        MemoryCommand::Pop(
                            PossibleCommands::memory_location(&line)?
                        )
                    )
                )
            },
            "push"=>{
                Ok(
                    PossibleCommands::Memory(
                        MemoryCommand::Push(
                            PossibleCommands::memory_location(&line)?
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
            _=>Err(()),
        }
    }
    #[inline]
    fn memory_location(line: &Vec<&str>)->Result<MemoryLocation,()>{
        if line.len() != 3{
            return Err(());
        }
        fn parse(num:&str)->Result<i16,()>{
            match num.parse(){
                Ok(valor)=>Ok(valor),
                Err(_)=>Err(()),
            }
                
        }
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
            _=>Err(()),
        }
    }
    pub fn to_asm(&self,file_name:&str)->String{
        match self{
            PossibleCommands::Memory(inner)=>{
                inner.to_asm(file_name)
            },
            PossibleCommands::Arithmetic(inner)=>{
                inner.to_asm()
            }
        }
    }
}
pub enum MemoryCommand{
    Pop(MemoryLocation),
    Push(MemoryLocation),
}
impl MemoryCommand{
    fn to_asm(&self,file_name:&str)->String{
        match self{
            MemoryCommand::Pop(value)=>{
                String::new()
            },
            MemoryCommand::Push(value)=>{
                //una vez que el valor a pushear esta en D, agregar esto
                let push_signature = "@SP\nA=M\nM=D\n@SP\nM=M+1\n";
                match value{
                    MemoryLocation::Constant(constante)=>{
                        return format!("@{}\nD=A\n{}",constante,push_signature)
                    },
                    MemoryLocation::Temp(direccion)=>{
                        if *direccion > 7{
                            panic!("Compilation error");
                        }
                        return format!("@5\nA=A+{}\nD=M\n{}",direccion,push_signature);
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
                        return format!("@{}\nA=M+{}\nD=M\n{}",seccion,numero,push_signature);
                    }
                    
                }
            }
        }
    }
}
pub enum MemoryLocation{
    Static(i16),
    Constant(i16),
    Local(i16),
    Temp(i16),
    Pointer(i16),
    Argument(i16),
    This(i16),
    That(i16),
}
pub enum ArithmeticCommand{
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
    fn to_asm(&self)->String{
        todo!();
    }
}
pub struct ComandosParseados{
    pub comandos: Vec<PossibleCommands>,
    pub comandos_str: Option<Vec<String>>,
}
impl ComandosParseados{
    #[inline]
    pub fn parse_commands(texto: String,verbose:bool)->Result<ComandosParseados,()>{
        let mut comandos = Vec::new();
        let mut comandos_str:Vec<String> = Vec::new();
        'outer:for line in texto.lines(){
            let line = match strip_command(line){
                Some(valor)=>valor,
                None=>continue 'outer,
            };
            if verbose{
                comandos_str.push(line.to_string());
            }
            let line = line.split_whitespace().collect::<Vec<&str>>();
            //Si contiene 0 elementos hay algo que no anda como debería, debe tener al menos 1
            comandos.push(PossibleCommands::parse_command(line)?);
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
}
#[inline]
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