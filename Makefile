all: vblocker

vblocker: vblocker.rs
	@echo Compiling [$^]
	@rustc $^
	
