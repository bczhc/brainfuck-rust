#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>

#define MF if (fflush(stdout) != 0) { abort(); }
#define M1(x) *ptr += x;
#define M2(x) ptr += x;
#define M3(x) c = getchar(); if (c != EOF) { *ptr = c; } else { *ptr = x; }
#define M4(x) x (*ptr);
#define M5 while (*ptr) {
#define M6 }

int getUTF8Size(uint32_t codepoint) {
    if (codepoint >= 0x0 && codepoint <= 0x7f) {
        return 1;
    } else if (codepoint >= 0x80 && codepoint <= 0x7ff) {
        return 2;
    } else if (codepoint >= 0x800 && codepoint <= 0xffff) {
        return 3;
    } else if (codepoint >= 0x10000 && codepoint <= 0x10ffff) {
        return 4;
    }
    return 0;
}

int unicode2UTF8(uint8_t *dest, uint32_t codepoint) {
    int utf8Size = getUTF8Size(codepoint);
    switch (utf8Size) {
        case 1:
            dest[0] = (uint8_t)(codepoint & 0b01111111U);
            break;
        case 2:
            dest[1] = (uint8_t)(0b10000000U | (codepoint & 0b00111111U));
            dest[0] = (uint8_t)(0b11000000U | ((codepoint >> 6U) & 0b00111111U));
            break;
        case 3:
            dest[2] = (uint8_t)(0b10000000U | (codepoint & 0b00111111U));
            dest[1] = (uint8_t)(0b10000000U | ((codepoint >> 6U) & 0b00111111U));
            dest[0] = (uint8_t)(0b11100000U | ((codepoint >> 12U) & 0b00111111U));
            break;
        case 4:
            dest[3] = (uint8_t)(0b10000000U | (codepoint & 0b00111111U));
            dest[2] = (uint8_t)(0b10000000U | ((codepoint >> 6U) & 0b00111111U));
            dest[1] = (uint8_t)(0b10000000U | ((codepoint >> 12U) & 0b00111111U));
            dest[0] = (uint8_t)(0b11110000U | ((codepoint >> 18U) & 0b00111111U));
            break;
        default:
            break;
    }
    return utf8Size;
}

void printU8(uint8_t a) {
    putchar(a);
    MF
}

void printU32(uint32_t a) {
    uint8_t buf[4];
    int size = unicode2UTF8(buf, a);
    for (int i = 0; i < size; ++i) {
        putchar(buf[i]);
    }
    MF
}

void printU16(uint16_t a) {
    if ((uint32_t) a < 0x10000) {
        printU32((uint32_t) a);
    } else {
        abort();
    }
}

void printU64(uint64_t a) {
    if (a < UINT32_MAX) {
        printU32((uint32_t) a);
    } else {
        abort();
    }
}
