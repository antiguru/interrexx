/* The library's main code. It runs once, after the directives of every file
   of the library are installed, and finishes the classes the interpreter
   defines natively with what this library defines in Rexx. The interpreter
   builds `.local` and String's class-side constants itself.

   The mixin classes of the native classes, in each one's inherit order
   (r[class.index.hierarchy]). */
.String~inherit(.Comparable)
.Message~inherit(.MessageNotification)
.Message~inherit(.AlarmNotification)
.Array~inherit(.OrderedCollection)
.List~inherit(.OrderedCollection)
.Queue~inherit(.OrderedCollection)
.Directory~inherit(.MapCollection)
.StringTable~inherit(.MapCollection)
.Stem~inherit(.MapCollection)
.Table~inherit(.MapCollection)
.IdentityTable~inherit(.MapCollection)
.Relation~inherit(.MapCollection)
.Bag~inherit(.MapCollection)
.Bag~inherit(.SetCollection)
.Set~inherit(.MapCollection)
.Set~inherit(.SetCollection)

/* The methods four native classes share with a non-public class of this
   library, copied into the native class's own instance methods rather than
   inherited, so the native class gains no superclass: the set operations of
   Set (r[class.set.operand]), of Bag (r[class.bag.operand]) and of Relation
   (r[class.relation.operand]), and the array readers of Supplier
   (r[class.supplier.allitems]). */
.Set~inheritInstanceMethods(.SetMixin)
.Bag~inheritInstanceMethods(.BagMixin)
.Relation~inheritInstanceMethods(.ManyItemMixin)
.Supplier~inheritInstanceMethods(.SupplierMixin)

/* Every public class this library declares, under its name in
   `.environment` (r[env.objects.entries]). The interpreter holds a place in
   `.environment` for each name, so the order of `.environment` does not
   depend on the order of these writes. */
public = .array~of( ,
  'Alarm', 'AlarmNotification', 'ArgUtil', 'CaselessColumnComparator', ,
  'CaselessComparator', 'CaselessDescendingComparator', 'CircularQueue', ,
  'Collection', 'ColumnComparator', 'CommandContext', 'CommandHandler', ,
  'CommandInput', 'CommandOutput', 'Comparable', 'Comparator', 'DateTime', ,
  'DescendingComparator', 'File', 'InputOutputStream', 'InputStream', ,
  'InvertingComparator', 'MapCollection', 'MessageNotification', 'Monitor', ,
  'NativeCommandHandler', 'NumericComparator', 'Orderable', ,
  'OrderedCollection', 'OutputStream', 'Properties', 'RexxQueue', ,
  'SetCollection', 'Singleton', 'Stream', 'StreamSupplier', 'Ticker', ,
  'TimeSpan', 'TraceObject', 'Validate')
do name over public
  .environment~setEntry(name, .context~package~findClass(name))
end
return
