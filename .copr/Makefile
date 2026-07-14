VERSION := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"//;s/"//')
ARCHIVE  := boulderX-$(VERSION).tar.gz
SPECFILE := packaging/boulderX.spec

.PHONY: srpm
srpm:
	git archive --prefix=boulderX-$(VERSION)/ HEAD | gzip > /tmp/$(ARCHIVE)
	cp /tmp/$(ARCHIVE) $(outdir)/
	cp $(SPECFILE) $(outdir)/boulderX.spec
	rpmbuild -bs \
	    --define "_sourcedir $(outdir)" \
	    --define "_srcrpmdir $(outdir)" \
	    --define "_specdir   $(outdir)" \
	    $(outdir)/boulderX.spec
