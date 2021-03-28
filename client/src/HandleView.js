import React, { Component } from 'react';
import { connect } from 'react-redux';

import Box from '@material-ui/core/Box';
import Button from '@material-ui/core/Button';
import Chip from '@material-ui/core/Chip';
import Grid from '@material-ui/core/Grid';
import Paper from '@material-ui/core/Paper';
import Typography from '@material-ui/core/Typography';
import List from '@material-ui/core/List';
import ListItem from '@material-ui/core/ListItem';
import LockTwoToneIcon from '@material-ui/icons/LockTwoTone';
import LockOpenTwoToneIcon from '@material-ui/icons/LockOpenTwoTone';
import VisibilityTwoToneIcon from '@material-ui/icons/VisibilityTwoTone';
import ThumbUpTwoToneIcon from '@material-ui/icons/ThumbUpTwoTone';
import ThumbDownTwoToneIcon from '@material-ui/icons/ThumbDownTwoTone';
import { withStyles } from '@material-ui/core/styles';

import FardelViewDialog from './FardelViewDialog';
import { queryGetFardelsFromHandle,
         executeFollow,
         executeUnpack,
         setViewedHandle,
         setFollowing,
         setFardelViewDialogOpen,
         setSelectedFardel,
       } from './store/Actions';

const useStyles = theme => ({
  button: {
    margin: theme.spacing(1),
  },
  buttonBox: {
    width: '100%',
  },
  root: {
    display: 'flex',
    flexWrap: 'wrap',
  },
  headerBox: {
    width: '100%',
    padding: theme.spacing(2),
  },
  paper: {
    padding: theme.spacing(2),
    margin: 'auto',
    width: '100%',
  },
  img: {
    margin: 'auto',
    display: 'block',
    maxWidth: 60,
    maxHeight: 60,
  },
});

class HandleView extends Component {

  handleGotoMyFardelsButtonClick() {
    const { handle, setViewedHandle, queryGetFardelsFromHandle } = this.props;

    setViewedHandle(handle);
    queryGetFardelsFromHandle(handle, 0, 10);
  }

  handleFollowButtonClick() {
    const { viewedHandle, executeFollow, following, setFollowing } = this.props;

    if (!following.includes(viewedHandle)) {
      executeFollow(viewedHandle);
      setFollowing([...following, viewedHandle]);
    }

  }

  handleUnpackButtonClick(fardel, idx) {
    const { executeUnpack, setSelectedFardel } = this.props;
    setSelectedFardel(idx);
    executeUnpack(fardel, idx);
  }

  handleViewButtonClick(fardelId, idx) {
    const { setFardelViewDialogOpen, setSelectedFardel } = this.props;

    setSelectedFardel(idx);
    console.log(idx);
    setFardelViewDialogOpen(true);
  }

  printDate(fardel) {
    return new Date(fardel.timestamp * 1000).toLocaleString("en-US");
  }

  render() {
  	const { classes, viewedFardels, viewedHandle, handle, loggedIn } = this.props;

    return (
      <div className={classes.root}>
        <FardelViewDialog />
        <Grid container spacing={1} alignItems={"center"}>
          <Grid item xs={4}>
            <Box className={classes.headerBox}>
              <Typography variant="h6">{"@"+viewedHandle}</Typography>
            </Box>
          </Grid>
          <Grid item xs={4}>
            <Box className={classes.buttonBox} display="flex" justifyContent="flex-start" p={1}>
              <Button
                variant="contained"
                color="default"
                className={classes.button}
                onClick={() => this.handleFollowButtonClick()}
              >
                Follow
              </Button>
            </Box>
          </Grid>
          <Grid item xs={4}>
            <Box className={classes.buttonBox} display="flex" justifyContent="flex-start" p={1}>
              <Button
                variant="contained"
                color="default"
                className={classes.button}
                onClick={() => this.handleGotoMyFardelsButtonClick()}
                disabled={!loggedIn}
              >
                Goto My Fardels
              </Button>
            </Box>
          </Grid>
        </Grid>
        <Grid item xs={11}>
        <List>
  	      {viewedFardels.map((fardel, index) => (
  	        <ListItem key={index}>
            <Paper className={classes.paper}>
              <Grid container spacing={2}>
                <Grid item xs={2}>
                  { fardel.img &&
                    <img className={classes.img} alt="thumbnail" src={fardel.img} />
                  }
                </Grid>
                <Grid item xs={6} sm container>
                  <Grid item xs container direction="column" spacing={2}>
                    <Grid item xs>
                      <Typography gutterBottom>
                        { fardel.packed
                          ? <LockTwoToneIcon />
                          : <VisibilityTwoToneIcon />
                        } 
                        {fardel.public_message}
                      </Typography>
                      <Typography variant="body2">
                        {this.printDate(fardel)}
                      </Typography>
                      <Typography>
                        {fardel.upvotes}<ThumbUpTwoToneIcon />, {fardel.downvotes}<ThumbDownTwoToneIcon />
                      </Typography>
                    </Grid>
                    { fardel.has_ipfs_cid &&
                        <Grid item>
                          <Chip label="IPFS" />
                        </Grid>
                    }
                  </Grid>
                </Grid>
                <Grid item xs={4} container>
                  <Grid item xs container direction="column" spacing={2}>
                    <Grid item xs>
                      <Typography variant="subtitle1">SCRT {fardel.cost}</Typography>
                    </Grid>
                    <Grid item xs>
                      <Box className={classes.buttonBox} display="flex" justifyContent="flex-start" p={1}>
                        { fardel.packed 
                          ? <Button
                              variant="contained"
                              color="default"
                              className={classes.button}
                              startIcon={<LockOpenTwoToneIcon />}
                              onClick={() => this.handleUnpackButtonClick(fardel, index)}
                              disabled={!fardel.packed}
                            >
                              Unpack
                            </Button>
                          : <Button
                              variant="contained"
                              color="default"
                              className={classes.button}
                              startIcon={<VisibilityTwoToneIcon />}
                              onClick={() => this.handleViewButtonClick(fardel.id, index)}
                              disabled={fardel.packed}
                            >
                              View
                            </Button>
                        }
                      </Box>
                    </Grid>
                  </Grid>
                </Grid>
              </Grid>
            </Paper>
            </ListItem>
          ))}
        </List>
        </Grid>
        
      </div>
    );
  }
}

const mapStateToProps = (state) => {
  return {
    keplrEnabled: state.keplrEnabled,
    viewedHandle: state.viewedHandle,
    viewedFardels: state.viewedFardels,
    loggedIn: state.loggedIn,
    handle: state.handle,
    following: state.following,
  }
};

const mapDispatchToProps = (dispatch) => {
  return ({
    executeUnpack: (fardel, idx) => { dispatch(executeUnpack(fardel, idx)) },
    executeFollow: (handle) => { dispatch(executeFollow(handle)) },
    queryGetFardelsFromHandle: (handle, page, pageSize) => 
        { dispatch(queryGetFardelsFromHandle(handle, page, pageSize)) },
    setViewedHandle: (viewedHandle) => { dispatch(setViewedHandle(viewedHandle)) },
    setFollowing: (following) => { dispatch(setFollowing(following)) },
    setFardelViewDialogOpen: (fardelViewDialogOpen) => 
        { dispatch(setFardelViewDialogOpen(fardelViewDialogOpen)) },
    setSelectedFardel: (selectedFardel) => { dispatch(setSelectedFardel(selectedFardel)) },
  });
}

export default connect(mapStateToProps, mapDispatchToProps)(withStyles(useStyles)(HandleView));
