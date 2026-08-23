/* integrators_test.c — C reference for the integrator cross-checks.
 * Two-body problem with explicit Cartesian initial conditions, fixed
 * particle data, no randomness, no pow() anywhere on the path.
 * Usage: integrators_test <integrator> [order] [steps]
 * Dumps the final state as raw bit patterns to state_c_final.txt.
 * Part of the rebound_rs port verification. GPL-3.0-or-later. */
#include "rebound.h"
#include "integrator_leapfrog.h"
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
    void* state = reb_simulation_set_integrator(r, integrator);
    if (strcmp(integrator,"leapfrog")==0){
        struct reb_integrator_leapfrog_state* lf = state;
        lf->order = order;
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
