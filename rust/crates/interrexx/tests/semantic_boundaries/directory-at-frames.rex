/* Entry callbacks see the Directory lookup that ran them. */
d = .directory~new
d~setMethod('present', .method~new('', "f = .context~stackFrames[2]; return f~type f~name"))
d~setMethod('unknown', .method~new('', "f = .context~stackFrames[2]; return f~type f~name arg(1)"))
say d['PRESENT']
say d~at('PRESENT')
say d['missing']
say d~at('missing')
