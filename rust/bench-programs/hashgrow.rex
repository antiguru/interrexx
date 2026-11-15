/* Hash-store growth: a Table built from empty to 200,000 string keys, and a
   second to 50,000 keys of a class whose HASHCODE answers its string's, so
   every insertion crosses the rebuild thresholds and the second also
   dispatches a user method per probe. Three passes. */
n = 3
do n
    t = .table~new
    do i = 1 to 200000
        t~put(i, 'k'i)
    end
    u = .table~new
    do i = 1 to 50000
        u~put(i, .key~new('k'i))
    end
end
say t~items u~items

::class key
::method init
    expose v
    use arg v
::method hashCode
    expose v
    return v~hashCode
