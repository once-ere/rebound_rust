/* integrators_test.c — C reference for the integrator cross-checks.
 * Two-body problem with explicit Cartesian initial conditions, fixed
 * particle data, no randomness, no pow() anywhere on the path.
 * Usage: integrators_test <integrator> [order] [steps]
 * Dumps the final state as raw bit patterns to state_c_final.txt.
 * Part of the rebound_rs port verification. GPL-3.0-or-later. */
#include "rebound.h"
#include "integrator_leapfrog.h"
#include "integrator_whfast.h"
#include "integrator_saba.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static unsigned long long bits(double x){
    unsigned long long u; memcpy(&u,&x,8); return u;
}

int main(int argc, char* argv[]){
    const char* integrator = argc>1 ? argv[1] : "ias15";
    unsigned int order = argc>2 ? (unsigned int)atoi(argv[2]) : 2;
    unsigned long long nsteps = argc>3 ? strtoull(argv[3],NULL,10) : 1000;

    struct reb_simulation* r = reb_simulation_create();
    /* whfast configurations are encoded as pseudo names:
     *   whfast        default (jacobi, safe_mode)
     *   whfast-c11    corrector 11        whfast-c17    corrector 17
     *   whfast-dh     democratic heliocentric
     *   whfast-whds   WHDS coordinates
     *   whfast-bary   barycentric coordinates
     *   whfast-mk     modified kick kernel
     *   whfast-comp   composition kernel
     *   whfast-lazy   lazy implementer's kernel
     *   whfast-usafe  safe_mode = 0 */
    const char* real_integrator = integrator;
    if (strncmp(integrator, "whfast", 6)==0) real_integrator = "whfast";
    if (strncmp(integrator, "saba", 4)==0) real_integrator = "saba";
    void* state = reb_simulation_set_integrator(r, real_integrator);
    if (strcmp(integrator,"leapfrog")==0){
        struct reb_integrator_leapfrog_state* lf = state;
        lf->order = order;
    }
    if (strncmp(integrator, "whfast", 6)==0){
        struct reb_integrator_whfast_state* wh = state;
        if (strcmp(integrator,"whfast-c11")==0)  wh->corrector = 11;
        if (strcmp(integrator,"whfast-c17")==0){ wh->corrector = 17; wh->corrector2 = 1; }
        if (strcmp(integrator,"whfast-dh")==0)   wh->coordinates = REB_INTEGRATOR_WHFAST_COORDINATES_DEMOCRATICHELIOCENTRIC;
        if (strcmp(integrator,"whfast-whds")==0) wh->coordinates = REB_INTEGRATOR_WHFAST_COORDINATES_WHDS;
        if (strcmp(integrator,"whfast-bary")==0) wh->coordinates = REB_INTEGRATOR_WHFAST_COORDINATES_BARYCENTRIC;
        if (strcmp(integrator,"whfast-mk")==0)   wh->kernel = REB_INTEGRATOR_WHFAST_KERNEL_MODIFIEDKICK;
        if (strcmp(integrator,"whfast-comp")==0) wh->kernel = REB_INTEGRATOR_WHFAST_KERNEL_COMPOSITION;
        if (strcmp(integrator,"whfast-lazy")==0) wh->kernel = REB_INTEGRATOR_WHFAST_KERNEL_LAZY;
        if (strcmp(integrator,"whfast-usafe")==0) wh->safe_mode = 0;
    }
    if (strncmp(integrator, "saba", 4)==0){
        struct reb_integrator_saba_state* sb = state;
        if (strcmp(integrator,"saba-1")==0)     sb->type = REB_INTEGRATOR_SABA_TYPE_1;
        if (strcmp(integrator,"saba-2")==0)     sb->type = REB_INTEGRATOR_SABA_TYPE_2;
        if (strcmp(integrator,"saba-3")==0)     sb->type = REB_INTEGRATOR_SABA_TYPE_3;
        if (strcmp(integrator,"saba-4")==0)     sb->type = REB_INTEGRATOR_SABA_TYPE_4;
        if (strcmp(integrator,"saba-cm2")==0)   sb->type = REB_INTEGRATOR_SABA_TYPE_CM_2;
        if (strcmp(integrator,"saba-cl2")==0)   sb->type = REB_INTEGRATOR_SABA_TYPE_CL_2;
        if (strcmp(integrator,"saba-104")==0)   sb->type = REB_INTEGRATOR_SABA_TYPE_10_4;
        if (strcmp(integrator,"saba-864")==0)   sb->type = REB_INTEGRATOR_SABA_TYPE_8_6_4;
        if (strcmp(integrator,"saba-h844")==0)  sb->type = REB_INTEGRATOR_SABA_TYPE_H_8_4_4;
        if (strcmp(integrator,"saba-h864")==0)  sb->type = REB_INTEGRATOR_SABA_TYPE_H_8_6_4;
        if (strcmp(integrator,"saba-h1064")==0) sb->type = REB_INTEGRATOR_SABA_TYPE_H_10_6_4;
        if (strcmp(integrator,"saba-usafe")==0) sb->safe_mode = 0;
    }
    r->G = 1.0;
    r->dt = 0.01;

    struct reb_particle star = {0};
    star.m = 1.0;
    reb_simulation_add(r, star);

    struct reb_particle planet = {0};
    planet.m = 1e-3;
    planet.x = 1.6;             /* apocenter of a=1, e=0.6 orbit */
    planet.vy = 0.5;            /* roughly the apocenter speed   */
    reb_simulation_add(r, planet);

    struct reb_particle moon = {0};
    moon.m = 1e-7;
    moon.x = 1.7;
    moon.vy = 0.6;
    moon.z = 0.01;
    moon.vz = 0.001;
    reb_simulation_add(r, moon);

    reb_simulation_steps(r, nsteps);

    FILE* f = fopen("state_c_final.txt","wb");
    fprintf(f, "integrator %s order %u steps %llu\n", integrator, order, nsteps);
    fprintf(f, "t %016llx\n", bits(r->t));
    fprintf(f, "dt %016llx\n", bits(r->dt));
    fprintf(f, "steps_done %llu\n", (unsigned long long)r->steps_done);
    for (size_t i=0;i<r->N;i++){
        struct reb_particle p = r->particles[i];
        fprintf(f, "%llu %016llx %016llx %016llx %016llx %016llx %016llx\n",
            (unsigned long long)i,
            bits(p.x), bits(p.y), bits(p.z),
            bits(p.vx), bits(p.vy), bits(p.vz));
    }
    fclose(f);
    printf("%s done: t=%.17e steps=%llu\n", integrator, r->t, (unsigned long long)r->steps_done);
    reb_simulation_free(r);
    return 0;
}
