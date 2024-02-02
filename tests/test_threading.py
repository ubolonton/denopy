import threading

import denopy


def test_not_segfault_on_many_runtime_objects():
    # Crashed Thread:        0  Dispatch queue: com.apple.main-thread
    # Exception Type:        EXC_BAD_ACCESS (SIGSEGV)
    # Exception Codes:       EXC_I386_GPFLT
    # Exception Note:        EXC_CORPSE_NOTIFY
    # Thread 0 Crashed:: Dispatch queue: com.apple.main-thread
    # 0   libsystem_kernel.dylib        	0x00007fff714c6b66 __pthread_kill + 10
    # 1   libsystem_pthread.dylib       	0x00007fff71691080 pthread_kill + 333
    # 2   libsystem_c.dylib             	0x00007fff713d46fe raise + 26
    # 3   libsystem_platform.dylib      	0x00007fff71684f5a _sigtramp + 26
    # 4   ???                           	000000000000000000 0 + 0
    # 5   denopy.abi3.so                	0x000000010e42bd6d v8::Isolate::New(v8::Isolate::CreateParams const&) + 29
    # 6   denopy.abi3.so                	0x000000010e3a2d9a v8::isolate::Isolate::new::h975afd50448ea9a5 + 106
    # 7   denopy.abi3.so                	0x000000010e2da9d1 deno_core::runtime::jsruntime::JsRuntime::new_inner::hb43acbbcdcf2c4f3 (.llvm.16324810852684978592) + 6593
    # 8   denopy.abi3.so                	0x000000010e2d89ef deno_core::runtime::jsruntime::JsRuntime::new::h1141a195ce54564a + 111
    # 9   denopy.abi3.so                	0x000000010e256721 denopy::_::_$LT$impl$u20$pyo3..impl_..pyclass..PyMethods$LT$denopy..Runtime$GT$$u20$for$u20$pyo3..impl_..pyclass..PyClassImplCollector$LT$denopy..Runtime$GT$$GT$::py_methods::ITEMS::trampoline::hfdca09eeb9cd145f (.llvm.9783906561352524097) + 801
    # 10  org.python.python             	0x000000010c94ba59 type_call + 46
    for i in range(20):
        # Seems like it crashes if there are many (how many?) references to Runtime objects on the stack.
        # Doesn't crash if the runtime:
        #   - Is not assigned to a variable.
        #   - Is assigned to a variable, but the variable is then deleted: `del r`.
        #   - Is added directly to a list without a variable: `runtimes.append(denopy.Runtime())`.
        r = denopy.Runtime()


def test_one_thread_per_runtime():
    runtime = denopy.Runtime()
    result = {}

    def _eval():
        try:
            runtime.eval("1")
        except BaseException as e:
            result['error'] = e

    thread = threading.Thread(target=_eval)
    thread.start()
    thread.join()
    # TODO: Figure out a way to get the exception type.
    assert "Runtime is unsendable, but sent to another thread" in str(result['error'])


def test_one_runtime_per_thread():
    runtime1 = denopy.Runtime()
    runtime2 = denopy.Runtime()
    assert runtime2 is runtime1
