#!/usr/bin/env perl
use strict;
use warnings;

# Advisory literal rendering, never TOML admission or release identity.
# Unsupported forms remain byte-for-byte unchanged. Callers own installation.
@ARGV == 3 or die "usage: render-cargo-readme-version.pl SOURCE PACKAGE CANDIDATE\n";
my ($path, $package, $candidate) = @ARGV;
$package =~ /\A[A-Za-z0-9][A-Za-z0-9_-]*\z/ or die "invalid package name\n";
$candidate =~ /\A(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\z/
    or die "expected canonical stable version\n";
-f $path && !-l $path or die "source must be a regular file\n";
open my $input, '<', $path or die "$path: $!\n";
binmode $input or die "$path: $!\n";
my $text = do { local $/; <$input> } // '';
close $input or die "$path: $!\n";

# Deliberately narrow: single-line strings without escapes, booleans, and
# string arrays cover ordinary Cargo dependency examples. Complex TOML is
# left to its parser rather than reconstructed here.
my $string = qr/(?:"[^"\\\r\n]*"|'[^'\r\n]*')/;
my $key = qr/(?:[A-Za-z0-9_-]+|"[A-Za-z0-9_-]+"|'[A-Za-z0-9_-]+')/;
my $array = qr/\[[ \t]*(?:$string[ \t]*(?:,[ \t]*$string[ \t]*)*,?[ \t]*)?\]/;
my $value = qr/(?:$string|true|false|$array)/;
my @matches;
my $offset = 0;
for my $line (split /(?<=\n)/, $text) {
    if ($line =~ /\A[ \t]*(?:\Q$package\E|"\Q$package\E"|'\Q$package\E')[ \t]*=[ \t]*/) {
        my $start = $+[0];
        my $rest = substr($line, $start);
        my $span;
        if ($rest =~ /\A($string)[ \t]*(?:\#[^\r\n]*)?\r?\n?\z/) {
            $span = [$offset + $start, length($1)];
        } elsif ($rest =~ /\A\{(.*)\}[ \t]*(?:\#[^\r\n]*)?\r?\n?\z/) {
            my $fields = $1;
            my %seen;
            my $valid = 1;
            pos($fields) = 0;
            while ($fields =~ /\G[ \t]*($key)[ \t]*=[ \t]*($value)[ \t]*(,|\z)/gc) {
                my ($name, $literal, $position, $separator) = ($1, $2, $-[2], $3);
                $name =~ s/\A["']|["']\z//g;
                $valid = 0 if $seen{$name}++;
                if ($name eq 'version') {
                    $valid = 0 unless $literal =~ /\A$string\z/;
                    $span = [$offset + $start + 1 + $position, length($literal)];
                }
                last if $separator eq '';
            }
            $valid = 0 unless substr($fields, pos($fields) // 0) =~ /\A[ \t]*\z/;
            $span = undef unless $valid;
        }
        push @matches, $span;
    }
    $offset += length($line);
}
if (@matches == 1 && defined $matches[0]) {
    substr($text, $matches[0][0], $matches[0][1], qq{"$candidate"});
} else {
    warn "Reminder: review $package dependency examples; no unique supported example; unchanged.\n";
}
binmode STDOUT or die "stdout: $!\n";
print $text or die "output: $!\n";
