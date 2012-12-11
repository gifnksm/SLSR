SRC=$(wildcard *.rs)
CRATE=slither.rc
TARGET=$(CRATE:.rc=)
TEST_TARGET=$(CRATE:.rc=.test)

RUSTC_FLAGS=-W non-camel-case-types --opt-level 3
LD_FLAGS=
TEST_RUSTC_FLAGS=-g

all: $(TARGET)

$(TARGET): $(CRATE) $(SRC)
	rustc $(RUSTC_FLAGS) $<

$(TEST_TARGET): $(CRATE) $(SRC)
	rustc --test $(RUSTC_FLAGS) $(TEST_RUSTC_FLAGS) $(LD_FLAGS) $(CRATE) -o $@

.PHONY: test clean

test: $(TEST_TARGET)
	./$(TEST_TARGET)

clean:
	$(RM) $(TARGET) $(TEST_TARGET)

