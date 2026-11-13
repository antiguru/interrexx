quiet_ = .output~destination(.stream~new('/dev/null'))
signal on syntax name raised_
q = rxqueue('Create', 'spec_rxqueue_exist')
say rxqueue('Exist', q) rxqueue('Exist', 'SESSION')
call rxqueue 'Delete', q
say rxqueue('Exist', q) rxqueue('Exist', 'spec_rxqueue_exist_none')

exit
raised_: c_ = condition('O'); .stdout~lineout(c_~code); do i_ = 1 to c_~additional~items; .stdout~lineout(i_ c_~additional[i_]); end
