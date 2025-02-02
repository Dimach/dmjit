#define CLEAR_CACHE_VAR neutral.nop()

/proc/do_test()
    compile_proc(/proc/receive_datum)
    compile_proc(/proc/access_datum)
    compile_proc(/proc/pass_datum)
    compile_proc(/proc/store_restore_datum)
    compile_proc(/proc/deopt_ret)
    compile_proc(/proc/deopt_arg)
    compile_proc(/datum/base/proc/deopt_src)
    compile_proc(/datum/base/proc/call_nested)
    compile_proc(/datum/base/proc/two_arg)
    compile_proc(/datum/base/proc/unbalanced_if)
    CHECK_INSTALL_COMPILED // RES: /receive_datum, /access_datum, /pass_datum, /store_restore_datum, /deopt_ret, /deopt_arg, /datum/base/deopt_src, /datum/base/call_nested, /datum/base/two_arg, /datum/base/unbalanced_if

    var/datum/base/dt_local = new
    var/datum/base/dt_local_two = new
    var/datum/base/neutral = new

    MARK_REF_COUNT(dt_local)
    MARK_REF_COUNT(dt_local_two)

    RES(CHECK_LEAK(dt_local)) // RES: OK

    receive_datum(dt_local)
    RES(CHECK_LEAK(dt_local)) // RES: OK

    access_datum(dt_local)
    RES(CHECK_LEAK(dt_local)) // RES: OK

    pass_datum(dt_local)
    RES(CHECK_LEAK(dt_local)) // RES: OK

    store_restore_datum(dt_local)
    RES(CHECK_LEAK(dt_local)) // RES: OK

    deopt_ret(dt_local)
    RES(CHECK_LEAK(dt_local)) // RES: OK

    deopt_arg(dt_local)
    RES(CHECK_LEAK(dt_local)) // RES: OK

    dt_local.deopt_src()
    CLEAR_CACHE_VAR
    RES(CHECK_LEAK(dt_local)) // RES: OK

    dispatch_call_nested(dt_local)
    RES(CHECK_LEAK(dt_local)) // RES: OK

    dispatch_call_two_arg(dt_local, dt_local_two)
    RES(CHECK_LEAK(dt_local)) // RES: OK
    RES(CHECK_LEAK(dt_local_two)) // RES: OK

    dt_local.unbalanced_if_wrap(TRUE)
    RES(CHECK_LEAK(dt_local)) // RES: OK

/datum/base
    var/dt_next = null

/datum/base/proc/nop()
    return

/proc/receive_datum(arg)
    return arg

/proc/access_datum(var/datum/base/arg)
    arg.dt_next = 1

/proc/pass_datum(arg)
    return just_ret(arg)

/proc/store_restore_datum(var/datum/base/arg)
    arg.dt_next = arg
    var/q = arg.dt_next
    arg.dt_next = 1
    return arg

/proc/just_ret(arg)
    return arg

/proc/dispatch_call_nested(var/datum/base/arg)
    return arg.call_nested()

/datum/base/proc/call_nested()
    return nested()

/datum/base/proc/nested()
    return 10

/proc/dispatch_call_two_arg(var/datum/base/one, var/datum/base/two)
    return one.two_arg(two)

/datum/base/proc/two_arg(var/datum/base/other)
    var/l = call_nested()
    dm_jitaux_deopt()
    return l + other.call_nested()

/datum/base/proc/unbalanced_if(v)
    if (v)
        var/datum/base/l = call_nested()
        l.nested()

/datum/base/proc/unbalanced_if_wrap(v)
    unbalanced_if(v)

/proc/deopt_ret(arg)
    var/l = arg
    dm_jitaux_deopt()
    return l

/proc/deopt_arg(arg)
    dm_jitaux_deopt()
    return arg

/datum/base/proc/deopt_src()
    dm_jitaux_deopt()
    src

// Intrinsic to emulate deopt
/proc/dm_jitaux_deopt()