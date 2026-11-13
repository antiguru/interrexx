/* A class reached only through another live class -- as its superclass, its
   INHERITed mixin or its metaclass -- survives a collection, and goes with
   that class once nothing holds either. */
before = .Object~subclasses~items

/* A superclass held only by a subclass whose instance is live. */
d = .Table~subclass('C1')~subclass('D1')
o = d~new
call gc 'force'
o[1] = 2
say 'superclass:' d~superclass~id o~isA(d~superclass) o[1]

/* A superclass held only by a subclass. */
b = .Object~subclass('A2')~subclass('B2')
call gc 'force'
say 'subclass only:' b~superclass~id b~new~isA(b~superclass)

/* A mixin held only through INHERIT. */
mx = .Object~mixinclass('MX3')
mx~define('GREET', 'return "greeting"')
y = .Object~subclass('Y3')
y~inherit(mx)
drop mx
call gc 'force'
say 'mixin:' y~new~greet y~superclasses~items

/* A metaclass held only by the class it made. */
m = .Class~subclass('M4')
m~define('HELLO', 'return "hello from" self~id')
x = .Object~subclass('X4', m)
drop m
call gc 'force'
say 'metaclass:' x~hello x~class~id

/* Dropping the holders frees the classes they kept. */
say 'held:' (.Object~subclasses~items - before)
drop d o b y x
call gc 'force'
say 'after drop:' (.Object~subclasses~items - before)
