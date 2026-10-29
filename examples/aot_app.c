/* Generated automatically by Parvan Native AOT Engine v4.0 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>
#include <math.h>

typedef int64_t prv_int;
typedef double prv_float;
typedef const char* prv_str;

void prv_print_int(prv_int x) { printf("%lld\n", (long long)x); }
void prv_print_str(prv_str s) { printf("%s\n", s); }
void prv_print_float(prv_float f) { printf("%g\n", f); }

int main(int argc, char** argv) {
    prv_int _u634_u631_u648_u639 = 1;
    prv_int _u62c_u645_u639 = 0;
    while ((_u634_u631_u648_u639 <= 10)) {
        _u62c_u645_u639 = (_u62c_u645_u639 + _u634_u631_u648_u639);
        _u634_u631_u648_u639 = (_u634_u631_u648_u639 + 1);
    }
    return 0;
}
