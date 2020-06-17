use rusty_v8 as v8;
extern crate libloading as lib;

pub fn main() {
  let platform = v8::new_default_platform().unwrap();
  v8::V8::initialize_platform(platform);
  v8::V8::initialize();
  let mut isolate = v8::Isolate::new(Default::default());
  let mut handle_scope = v8::HandleScope::new(&mut isolate);
  let scope = handle_scope.enter();

  let object_templ = v8::ObjectTemplate::new(scope);
  let function_templ = v8::FunctionTemplate::new(scope, core_instantiate_async);
  let name = v8::String::new(scope, "coreInstantiateAsync").unwrap();
  object_templ.set(name.into(), function_templ.into());

  let context = v8::Context::new_from_template(scope, object_templ);
  let mut cs = v8::ContextScope::new(scope, context);
  let scope = cs.enter();

  let code = v8::String::new(scope, "coreInstantiateAsync('adder');").unwrap();

  let mut script = v8::Script::compile(scope, context, code, None).unwrap();
  let result = script.run(scope, context).unwrap();
  let result = result.to_string(scope).unwrap();
  println!("result: {}", result.to_rust_string_lossy(scope));
}

pub fn core_instantiate_async(
    scope: v8::FunctionCallbackScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let context = scope.get_current_context().unwrap();
    let resolver = v8::PromiseResolver::new(scope, context).unwrap();
    let promise = resolver.get_promise(scope);

    let mut resolver_handle = v8::Global::new();
    resolver_handle.set(scope, resolver);
    {
        instantiate_async(
            scope,
            context,
            resolver_handle,
        );
    }

    rv.set(promise.into());
}

type RunAsyncFunc = unsafe fn(cb: Box<dyn FnMut(Option<String>)>);

pub fn instantiate_async<'a>(
    scope: &mut impl v8::ToLocal<'a>,
    context: v8::Local<'a, v8::Context>,
    mut resolver_handle: v8::Global<v8::PromiseResolver>,
) {
    let resolver = resolver_handle.get(scope).unwrap();
    resolver_handle.reset(scope);
    let cb = |response: Option<String>| {
        if let Some(res) = response {
            let value = v8::String::new(scope, &res).unwrap();
            resolver.resolve(context, value.into()).unwrap();
        } else {
            resolver
               .resolve(context, v8::undefined(scope).into())
               .unwrap();
        }
    };

    let plugin = lib::Library::new("./my_lib.so").unwrap();
    let run_async: lib::Symbol<RunAsyncFunc> = plugin.get(b"run_async").unwrap();
    run_async(Box::new(cb));
}
