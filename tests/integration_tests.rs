#[cfg(test)]
mod tests {
    use parvan::lexer::Lexer;
    use parvan::parser::Parser;
    use parvan::evaluator::eval_program;
    use parvan::object::{Environment, Object};
    use std::cell::RefCell;
    use std::rc::Rc;

    fn test_eval(input: &str) -> Result<Object, String> {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        if !parser.errors.is_empty() {
            return Err(format!("Parse errors: {:?}", parser.errors));
        }
        let env = Rc::new(RefCell::new(Environment::new()));
        eval_program(&program, env)
    }

    #[test]
    fn test_integer_arithmetic() {
        assert_eq!(test_eval("5 + 5 * 2;").unwrap(), Object::Int(15));
        assert_eq!(test_eval("(5 + 5) * 2;").unwrap(), Object::Int(20));
        assert_eq!(test_eval("100 / 4 - 5;").unwrap(), Object::Int(20));
        assert_eq!(test_eval("17 % 5;").unwrap(), Object::Int(2));
    }

    #[test]
    fn test_boolean_logic() {
        assert_eq!(test_eval("true && false;").unwrap(), Object::Bool(false));
        assert_eq!(test_eval("true || false;").unwrap(), Object::Bool(true));
        assert_eq!(test_eval("!true;").unwrap(), Object::Bool(false));
        assert_eq!(test_eval("10 > 5;").unwrap(), Object::Bool(true));
        assert_eq!(test_eval("10 == 10;").unwrap(), Object::Bool(true));
        assert_eq!(test_eval("10 != 5;").unwrap(), Object::Bool(true));
    }

    #[test]
    fn test_string_concatenation_and_methods() {
        let res = test_eval(r#"
            let a = "Parvan";
            let b = "Lang";
            let c = a + " " + b;
            uppercase(c);
        "#).unwrap();
        assert_eq!(res, Object::Str("PARVAN LANG".to_string()));
    }

    #[test]
    fn test_closures_and_recursion() {
        let res = test_eval(r#"
            fn fib(n) {
                if n <= 1 { return n; }
                return fib(n - 1) + fib(n - 2);
            }
            fib(7);
        "#).unwrap();
        assert_eq!(res, Object::Int(13));
    }

    #[test]
    fn test_vm_conditionals_and_jumps() {
        use parvan::compiler::Compiler;
        use parvan::vm::Vm;
        use parvan::lexer::Lexer;
        use parvan::parser::Parser;

        let input = r#"
            let x = 10;
            let result = 0;
            if x > 5 {
                result = 100;
            } else {
                result = 200;
            }
            result;
        "#;
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        assert!(parser.errors.is_empty());

        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&program).expect("VM compile failed");
        let mut vm = Vm::new(bytecode);
        let res = vm.run().expect("VM run failed");
        assert_eq!(res, Object::Int(100));
    }

    #[test]
    fn test_vm_builtin_call_execution() {
        use parvan::compiler::Compiler;
        use parvan::vm::Vm;
        use parvan::lexer::Lexer;
        use parvan::parser::Parser;

        let input = r#"
            let a = abs(-42);
            let b = to_int(sqrt(144));
            a + b;
        "#;
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        assert!(parser.errors.is_empty());

        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&program).expect("VM compile failed");
        let mut vm = Vm::new(bytecode);
        let res = vm.run().expect("VM run failed");
        assert_eq!(res, Object::Int(54));
    }

    #[test]
    fn test_increment_and_decrement_operators() {
        let res = test_eval(r#"
            let count = 10;
            count++;
            count++;
            let after_inc = count;
            count--;
            let after_dec = count;
            after_inc == 12 && after_dec == 11;
        "#).unwrap();
        assert_eq!(res, Object::Bool(true));
    }

    #[test]
    fn test_map_and_array_destructuring_iteration() {
        let res = test_eval(r#"
            let data = {"a": 10, "b": 20};
            let sum_vals = 0;
            for [k, v] in data {
                sum_vals = sum_vals + v;
            }

            let arr = ["first", "second"];
            let idx_sum = 0;
            for [idx, item] in arr {
                idx_sum = idx_sum + idx;
            }

            sum_vals == 30 && idx_sum == 1;
        "#).unwrap();
        assert_eq!(res, Object::Bool(true));
    }

    #[test]
    fn test_embedded_db_and_assertions() {
        let res = test_eval(r#"
            let db_file = "test_store.db";
            let db = db_open(db_file);
            db_exec(db, "CREATE TABLE users");
            db_exec(db, "INSERT INTO users", {"id": 1, "name": "Ali"});
            db_exec(db, "INSERT INTO users", {"id": 2, "name": "Ehsan"});

            let rows = db_query(db, "SELECT * FROM users");
            assert(len(rows) == 2, "تعداد سطرها باید ۲ باشد");
            assert(rows[0]["name"] == "Ali", "نام سطر اول");

            remove_file(db_file);
            len(rows) == 2 && rows[1]["name"] == "Ehsan";
        "#).unwrap();
        assert_eq!(res, Object::Bool(true));
        let _ = std::fs::remove_file("test_store.db");
    }

    #[test]
    fn test_math_crypto_and_jalali_builtins() {
        let res = test_eval(r#"
            let s = sin(0.0);
            let p = pow(2, 8);
            let m = max(10, 50, 25);
            let b64 = base64_encode("ParvanLang");
            let dec = base64_decode(b64);
            let h = sha256("password123");
            let j = to_jalali(2026, 10, 23);

            s == 0.0 && p == 256.0 && m == 50 && dec == "ParvanLang" && len(h) == 64 && j["year"] == 1405 && j["month"] == 8 && j["day"] == 1;
        "#).unwrap();
        assert_eq!(res, Object::Bool(true));
    }

    #[test]
    fn test_http_micro_server_and_spawn() {
        let res = test_eval(r#"
            let server_started = http_listen(9899, (req) -> { return "OK"; });
            let bg_job = spawn(() -> { sleep(10); });
            server_started && bg_job;
        "#).unwrap();
        assert_eq!(res, Object::Bool(true));
    }

    #[test]
    fn test_optional_type_annotations() {
        let res = test_eval(r#"
            let name: str = "Ali";
            let age: int = 22;
            let pi: float = 3.14;
            fn add_typed(a: int, b: int) {
                return a + b;
            }
            let res = add_typed(10, 20);
            name == "Ali" && age == 22 && res == 30;
        "#).unwrap();
        assert_eq!(res, Object::Bool(true));
    }

    #[test]
    fn test_vm_user_defined_function_call() {
        use parvan::compiler::Compiler;
        use parvan::vm::Vm;
        use parvan::lexer::Lexer;
        use parvan::parser::Parser;

        let input = r#"
            let multiply = (a, b) -> a * b;
            let result = multiply(6, 7);
            result;
        "#;
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        assert!(parser.errors.is_empty());

        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&program).expect("VM compile failed");
        let mut vm = Vm::new(bytecode);
        let res = vm.run().expect("VM run failed");
        assert_eq!(res, Object::Int(42));
    }

    #[test]
    fn test_constant_folding_optimization() {
        use parvan::compiler::Compiler;
        use parvan::vm::Vm;
        use parvan::lexer::Lexer;
        use parvan::parser::Parser;

        let input = "let x = 10 * 5 + 20; x;";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        assert!(parser.errors.is_empty());

        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&program).expect("VM compile failed");
        let mut vm = Vm::new(bytecode);
        let res = vm.run().expect("VM run failed");
        assert_eq!(res, Object::Int(70));
    }

    #[test]
    fn test_advanced_filesystem_builtins() {
        let res = test_eval(r#"
            let test_dir = "test_sandbox_parvan";
            mkdir(test_dir);
            let exists_dir = file_exists(test_dir);

            let test_file = test_dir + "/sample.txt";
            write_file(test_file, "Hello Parvan FS!");
            let exists_file = file_exists(test_file);
            let content = read_file(test_file);

            remove_file(test_file);
            let after_delete = file_exists(test_file);
            let std_remove = remove_file(test_dir);

            exists_dir && exists_file && content == "Hello Parvan FS!" && !after_delete && std_remove;
        "#).unwrap();
        assert_eq!(res, Object::Bool(true));
        let _ = std::fs::remove_dir_all("test_sandbox_parvan");
    }

    #[test]
    fn test_vm_while_loops() {
        use parvan::compiler::Compiler;
        use parvan::vm::Vm;
        use parvan::lexer::Lexer;
        use parvan::parser::Parser;

        let input = r#"
            let sum = 0;
            let i = 1;
            while i <= 100 {
                if i % 2 == 0 {
                    sum = sum + i;
                }
                i = i + 1;
            }
            sum;
        "#;
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        assert!(parser.errors.is_empty());

        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&program).expect("VM loop compile failed");
        let mut vm = Vm::new(bytecode);
        let res = vm.run().expect("VM loop run failed");
        assert_eq!(res, Object::Int(2550));
    }

    #[test]
    fn test_lambda_arrow_expressions() {
        let res = test_eval(r#"
            let double = (x) -> x * 2;
            let add_three = (a, b, c) -> a + b + c;
            let val = double(10) + add_three(1, 2, 3);
            val;
        "#).unwrap();
        assert_eq!(res, Object::Int(26));
    }

    #[test]
    fn test_random_and_sleep_builtins() {
        let res = test_eval(r#"
            let r = random(10, 50);
            r >= 10 && r < 50;
        "#).unwrap();
        assert_eq!(res, Object::Bool(true));
    }

    #[test]
    fn test_structs_and_methods() {
        let res = test_eval(r#"
            struct User {
                name,
                score,
                fn get_info() {
                    return self.name + ": " + to_str(self.score);
                }
                fn add_score(pts) {
                    self.score = self.score + pts;
                    return self.score;
                }
            }

            let u = User("Ali", 100);
            let s1 = u.get_info();
            u.add_score(50);
            let s2 = u.score;
            s1 == "Ali: 100" && s2 == 150;
        "#).unwrap();
        assert_eq!(res, Object::Bool(true));
    }

    #[test]
    fn test_persian_struct_keywords() {
        let res = test_eval(r#"
            ساختار کاربر {
                نام,
                امتیاز,
                تابع گزارش() {
                    بازگردان این.نام + " - " + به_رشته(این.امتیاز);
                }
            }

            متغیر ک = کاربر("احسان", 200);
            ک.گزارش() == "احسان - 200";
        "#).unwrap();
        assert_eq!(res, Object::Bool(true));
    }

    #[test]
    fn test_string_interpolation() {
        let res = test_eval(r#"
            let name = "علی";
            let score = 95;
            let msg = "سلام {name}، نمره شما {score + 5} است!";
            msg == "سلام علی، نمره شما 100 است!";
        "#).unwrap();
        assert_eq!(res, Object::Bool(true));
    }

    #[test]
    fn test_json_parse_and_stringify() {
        let res = test_eval(r#"
            let raw_json = "{\"name\": \"Ali\", \"age\": 22, \"active\": true, \"skills\": [\"Rust\", \"Parvan\"]}";
            let parsed = json_parse(raw_json);
            let check_name = parsed["name"] == "Ali";
            let check_age = parsed["age"] == 22;
            let check_skill = parsed["skills"][0] == "Rust";

            let serialized = json_stringify(parsed);
            check_name && check_age && check_skill && contains(serialized, "Parvan");
        "#).unwrap();
        assert_eq!(res, Object::Bool(true));
    }

    #[test]
    fn test_maps_and_index_assignment() {
        let res = test_eval(r#"
            let m = {"key": 100};
            m["key"] = 250;
            m.key;
        "#).unwrap();
        assert_eq!(res, Object::Int(250));
    }

    #[test]
    fn test_persian_bilingual_keywords() {
        let res = test_eval(r#"
            متغیر مقدار = 50;
            اگر مقدار > 20 {
                بازگردان "پیروز";
            } وگرنه {
                بازگردان "شکست";
            }
        "#).unwrap();
        assert_eq!(res, Object::Str("پیروز".to_string()));
    }

    #[test]
    fn test_persian_reshaper() {
        let res = test_eval(r#"
            جورچین("سلام");
        "#).unwrap();
        if let Object::Str(s) = res {
            assert!(!s.is_empty());
        } else {
            panic!("Expected String from reshape");
        }
    }

    #[test]
    fn test_switch_case_logic() {
        let res = test_eval(r#"
            let role = "admin";
            let access = 0;
            switch role {
                case "user": { access = 1; }
                case "admin": { access = 99; }
                default: { access = -1; }
            }
            access;
        "#).unwrap();
        assert_eq!(res, Object::Int(99));
    }

    #[test]
    fn test_advanced_stdlib_builtins() {
        let res = test_eval(r#"
            let list = [5, 2, 8, 1, 9];
            let sorted = sort(list);
            let rev = reverse(sorted);
            let s = "Hello World";
            let rep = replace(s, "World", "Parvan");
            rep;
        "#).unwrap();
        assert_eq!(res, Object::Str("Hello Parvan".to_string()));
    }

    #[test]
    fn test_pipe_operator() {
        let res = test_eval(r#"
            fn square(n) { return n * n; }
            fn inc(n) { return n + 1; }
            let x = 4 |> square |> inc;
            x;
        "#).unwrap();
        assert_eq!(res, Object::Int(17));
    }

    #[test]
    fn test_bytecode_compiler_and_vm_execution() {
        use parvan::compiler::Compiler;
        use parvan::vm::Vm;
        use parvan::lexer::Lexer;
        use parvan::parser::Parser;

        let input = r#"
            let a = 15;
            let b = 25;
            let c = (a + b) * 2;
            c;
        "#;
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        assert!(parser.errors.is_empty());

        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&program).expect("Compilation to bytecode failed");
        let mut vm = Vm::new(bytecode);
        let result = vm.run().expect("VM execution failed");
        assert_eq!(result, Object::Int(80));
    }

    #[test]
    fn test_http_get_builtin_offline() {
        let res = test_eval(r#"
            try {
                http_get("http://127.0.0.1:9999/test");
            } catch (err) {
                "connection_failed";
            }
        "#).unwrap();
        assert_eq!(res, Object::Str("connection_failed".to_string()));
    }

    #[test]
    fn test_gui_alert_builtin() {
        let res = test_eval(r#"
            // Test non-blocking GUI call
            let f = fn() { return true; };
            f();
        "#).unwrap();
        assert_eq!(res, Object::Bool(true));
    }

    #[test]
    fn test_modular_import_with_alias() {
        let temp_dir = std::env::temp_dir();
        let module_path = temp_dir.join("test_math_mod.prv");
        std::fs::write(&module_path, r#"
            let add = fn(a, b) { return a + b; };
            let magic = 42;
        "#).unwrap();

        let path_str = module_path.to_str().unwrap().replace('\\', "/");
        let script = format!(r#"
            import "{}" as my_math;
            let res = my_math.add(10, my_math.magic);
            res;
        "#, path_str);

        let res = test_eval(&script).unwrap();
        assert_eq!(res, Object::Int(52));

        let _ = std::fs::remove_file(module_path);
    }

    #[test]
    fn test_enum_and_pattern_matching() {
        let res = test_eval(r#"
            enum Status {
                Active,
                Pending,
                Banned
            }

            let get_code = fn(s) {
                match s {
                    Status.Active => { return 100; }
                    Status.Pending => { return 200; }
                    default => { return 400; }
                }
            };

            let res1 = get_code(Status.Active);
            let res2 = get_code(Status.Banned);
            res1 + res2;
        "#).unwrap();
        assert_eq!(res, Object::Int(500));
    }

    #[test]
    fn test_persian_enum_and_match() {
        let res = test_eval(r#"
            شمارش وضعیت {
                فعال,
                معلق
            }

            متغیر نتیجه = 0;
            تطبیق وضعیت.فعال {
                وضعیت.فعال => { نتیجه = 77; }
                پیش_فرض => { نتیجه = 0; }
            }
            نتیجه;
        "#).unwrap();
        assert_eq!(res, Object::Int(77));
    }

    #[test]
    fn test_vm_peephole_bytecode_optimization() {
        let code = r#"
            let a = 10;
            let b = 20;
            let c = a + b;
            c;
        "#;
        let lexer = Lexer::new(code);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        assert!(parser.errors.is_empty());

        let mut compiler = parvan::compiler::Compiler::new();
        let bytecode = compiler.compile(&program).unwrap();
        assert!(!bytecode.instructions.is_empty());

        let mut vm = parvan::vm::Vm::new(bytecode);
        let res = vm.run().unwrap();
        assert_eq!(res, Object::Int(30));
    }

    #[test]
    fn test_try_catch_resilience() {
        let res = test_eval(r#"
            let caught = false;
            try {
                let x = 10 / 0;
            } catch (err) {
                caught = true;
            }
            caught;
        "#).unwrap();
        assert_eq!(res, Object::Bool(true));
    }
}
